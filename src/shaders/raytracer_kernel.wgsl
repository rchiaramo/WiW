const EPSILON = 0.001f;

const PI = 3.1415927f;
const FRAC_1_PI = 0.31830987f;
const FRAC_PI_2 = 1.5707964f;

struct Sphere {
    center: vec4f,
    radius: f32,
    mat_idx: u32,
}

struct Material {
    albedo: vec4f,
    fuzz: f32,
    refract_idx: f32,
    mat_type: u32
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct HitPayload {
    t: f32,
    p: vec3f,
    n: vec3f,
    idx: u32,
}

struct CameraData {
    pos: vec3f,
    forwards: vec3f,
    right: vec3f,
    up: vec3f,
}

struct SamplingParameters {
    samples_per_pixel: u32,
    num_bounces: u32
}

@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> camera: CameraData;
@group(0) @binding(2) var<uniform> sampling_parameters: SamplingParameters;
@group(0) @binding(3) var<storage, read> spheres: array<Sphere>;
@group(0) @binding(4) var<storage, read> materials: array<Material>;
@compute @workgroup_size(1,1,1)
fn main(@builtin(global_invocation_id) id: vec3u) {

    let image_size: vec2<u32> = textureDimensions(color_buffer);
    let screen_pos = id.xy;

    let viewport_height: f32 = 2.0;
    let viewport_width: f32 = viewport_height * (f32(image_size.x) / f32(image_size.y));
    // need to change this when the camera starts moving...
    let viewport_u: vec3f = vec3f(viewport_width, 0.0, 0.0);
    let viewport_v: vec3f = vec3f(0.0, -viewport_height, 0.0);

    let du: vec3f = vec3f(viewport_width / f32(image_size.x), 0.0, 0.0);
    let dv: vec3f = vec3f(0.0, -viewport_height / f32(image_size.y), 0.0);

    let upper_left: vec3f = camera.pos + camera.forwards - viewport_u / 2.0 - viewport_v / 2.0;
    let pixel_00: vec3f = upper_left + du/2.0 + dv/2.0;

    // start here with main loop; for this position, loop over samples_per_pixel
    var pixel_color: vec3f = vec3f(0.0, 0.0, 0.0);
    var rng_state:u32 = initRng(screen_pos, image_size, 1u);
    for (var i: u32 = 0; i < sampling_parameters.samples_per_pixel; i++) {
        var ray: Ray = getRay(pixel_00, id.x, id.y, du, dv, &rng_state);
        pixel_color += rayColor(ray, &rng_state);
    }
    pixel_color = pixel_color / f32(sampling_parameters.samples_per_pixel);
//    pixel_color = vec3f(0.0, 0.0, 1.0);
//    pixel_color = vec3f(rngNextFloat(&rng_state), rngNextFloat(&rng_state), rngNextFloat(&rng_state));
//    let pixel_color: vec3<f32> = vec3<f32>(f32(screen_pos.x) / f32(screen_size.x), f32(screen_pos.y) / f32(screen_size.y), 0.0);

    textureStore(color_buffer, screen_pos, vec4<f32>(pixel_color, 1.0));
}

fn rayColor(primaryRay: Ray, state: ptr<function, u32>) -> vec3<f32> {
    // for every ray, we want to trace the ray through num_bounces
    // rayColor calls traceRay to get a hit, then calls it again
    // with new bounce ray
    var nextRay = primaryRay;
    var throughput: vec3f = vec3f(1.0);
    var pixel_color: vec3f = vec3f(0.0);
    for (var i: u32 = 0; i < sampling_parameters.num_bounces; i++) {
        var payLoad = HitPayload();

        if TraceRay(nextRay, &payLoad) {
            // depending on what kind of material, I need to find the scatter ray and the attenuation
            let mat_idx:u32 = spheres[payLoad.idx].mat_idx;
            let scatterRay: Ray = getScatterRay(nextRay, mat_idx, &payLoad, state);

            throughput *= materials[mat_idx].albedo.xyz;
            nextRay = scatterRay;
        } else {
            let a: f32 = 0.5 * (primaryRay.direction.y + 1.0);
            pixel_color = throughput * ((1.0 - a) * vec3f(1.0, 1.0, 1.0) + a * vec3f(0.5, 0.7, 1.0));
            break;
        }
    }

    return pixel_color;
}

fn TraceRay(ray: Ray, hit: ptr<function, HitPayload>) -> bool {
    // runs through objects in the scene and returns true if the ray hits one, and updates
    // the hitPayload with the closest hit

    var nearest_hit: f32 = 99999;
    let sphere_count = arrayLength(&spheres);
    var tempHitPayload = HitPayload();

    for (var i: u32 = 0; i < sphere_count; i++) {
        var newHitPayload = HitPayload();

        // I could update this code so that hit only determines if a hit happened and, if it did,
        // modifies the nearest_hit_t and stores the nearest_index
        if hit(ray, i, 0.001, nearest_hit, &newHitPayload) {
            nearest_hit = newHitPayload.t;
            tempHitPayload = newHitPayload;
        }
    }
    // then after looping through the objects, we will know the nearest_hit_t and the index; we could call
    // for the payload then (as opposed to filling it out every time we hit a closer sphere)
    if nearest_hit < 9999 {
        *hit = tempHitPayload;
        return true;
    }

    return false;
}

fn hit(ray: Ray, sphereIdx: u32, t_min: f32, t_nearest: f32, payload: ptr<function, HitPayload>) -> bool {
    // checks if the ray intersects the sphere given by sphereIdx; if so, returns true and modifies
    // a hitPayload to give the details of the hit
    let sphere: Sphere = spheres[sphereIdx];
    let sphere_center = sphere.center.xyz;
    let a: f32 = dot(ray.direction, ray.direction);
    let b: f32 = dot(ray.direction, ray.origin - sphere_center);
    let c: f32 = dot(ray.origin - sphere_center, ray.origin - sphere_center) -
        sphere.radius * sphere.radius;
    let discrim: f32 = b * b - a * c;


    if (discrim >= 0) {
        var t: f32 = (-b - sqrt(discrim)) / a;
        if (t > t_min && t < t_nearest) {
            *payload = hitSphere(t, ray, sphere, sphereIdx);
            return true;
        }

        t = (-b + sqrt(discrim)) / a;
        if (t > t_min && t < t_nearest) {
            *payload = hitSphere(t, ray, sphere, sphereIdx);
            return true;
        }
    }
    return false;
}
// need to add attenuation here
fn hitSphere(t: f32, ray: Ray, sphere: Sphere, idx: u32) -> HitPayload {
    // make the hitPayload struct
    let p: vec3f = ray.origin + t * ray.direction;
    let n: vec3f = normalize(p - sphere.center.xyz);

    return HitPayload(t, p, n, idx);
}

fn getRay(pixel_00: vec3f, x: u32, y: u32, du: vec3f, dv: vec3f, state: ptr<function, u32>) -> Ray {
    let offset: vec3f = rngNextVec3InUnitDisk(state);

    var ray: Ray;
    ray.origin = camera.pos + offset.x * du + offset.y * dv;
    ray.direction = normalize(pixel_00 + f32(x) * du + f32(y) * dv - camera.pos);
    return ray;
}

fn getScatterRay(inRay: Ray, mat_idx: u32, hit: ptr<function, HitPayload>, state: ptr<function, u32>) -> Ray {
    let payLoad = *hit;
    var ray = Ray();
    ray.origin = payLoad.p + 0.0001 * payLoad.n;

    let mat_type: u32 = materials[mat_idx].mat_type;

    switch (mat_type) {
        case 0u, default {
            var randomBounce: vec3f = normalize(rngNextVec3InUnitSphere(state));
            if dot(randomBounce, payLoad.n) < 0.0 {
                randomBounce *= -1.0;
            }
            // TODO! need to do something about a random bounce in opposite direction of normal
            ray.direction = payLoad.n + randomBounce;
        }
        case 1u {
            var randomBounce: vec3f = normalize(rngNextVec3InUnitSphere(state));
            let fuzz: f32 = materials[mat_idx].fuzz;
            ray.direction = reflect(inRay.direction, payLoad.n) + fuzz * randomBounce;
        }
        case 2u {

        }
    }

    return ray;
}

fn reflect(r: vec3f, n: vec3f) -> vec3f {
    return r - 2.0 * dot(r,n) * n;
}

fn rngNextInUnitHemisphere(state: ptr<function, u32>) -> vec3<f32> {
    let r1 = rngNextFloat(state);
    let r2 = rngNextFloat(state);

    let phi = 2.0 * PI * r1;
    let sinTheta = sqrt(1.0 - r2 * r2);

    let x = cos(phi) * sinTheta;
    let y = sin(phi) * sinTheta;
    let z = r2;

    return vec3(x, y, z);
}

fn rngNextVec3InUnitDisk(state: ptr<function, u32>) -> vec3<f32> {
    // Generate numbers uniformly in a disk:
    // https://stats.stackexchange.com/a/481559

    // r^2 is distributed as U(0, 1).
    let r = sqrt(rngNextFloat(state));
    let alpha = 2.0 * PI * rngNextFloat(state);

    let x = r * cos(alpha);
    let y = r * sin(alpha);

    return vec3(x, y, 0.0);
}

fn rngNextVec3InUnitSphere(state: ptr<function, u32>) -> vec3<f32> {
    // probability density is uniformly distributed over r^3
    let r = pow(rngNextFloat(state), 0.33333f);
    let theta = PI * rngNextFloat(state);
    let phi = 2.0 * PI * rngNextFloat(state);

    let x = r * sin(theta) * cos(phi);
    let y = r * sin(theta) * sin(phi);
    let z = r * cos(theta);

    return vec3(x, y, z);
}

fn rngNextUintInRange(state: ptr<function, u32>, min: u32, max: u32) -> u32 {
    rngNextInt(state);
    return min + (*state) % (max - min);
}

fn rngNextFloat(state: ptr<function, u32>) -> f32 {
    rngNextInt(state);
    return f32(*state) / f32(0xffffffffu);
}

fn initRng(pixel: vec2<u32>, resolution: vec2<u32>, frame: u32) -> u32 {
    // Adapted from https://github.com/boksajak/referencePT
    let seed = dot(pixel, vec2<u32>(1u, resolution.x)) ^ jenkinsHash(frame);
    return jenkinsHash(seed);
}

fn rngNextInt(state: ptr<function, u32>) {
    // PCG random number generator
    // Based on https://www.shadertoy.com/view/XlGcRh

    let oldState = *state + 747796405u + 2891336453u;
    let word = ((oldState >> ((oldState >> 28u) + 4u)) ^ oldState) * 277803737u;
    *state = (word >> 22u) ^ word;
}

fn jenkinsHash(input: u32) -> u32 {
    var x = input;
    x += x << 10u;
    x ^= x >> 6u;
    x += x << 3u;
    x ^= x >> 11u;
    x += x << 15u;
    return x;
}