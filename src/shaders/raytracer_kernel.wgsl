const EPSILON = 0.001f;

const PI = 3.1415927f;
const FRAC_1_PI = 0.31830987f;
const FRAC_PI_2 = 1.5707964f;

struct Sphere {
    center: vec3<f32>,
    albedo: vec3<f32>,
    radius: f32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct HitPayload {
    t: f32,
    p: vec3f,
    n: vec3f,
    color: vec3f,
    idx: u32,
    hit: bool,
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
    var multiplier: vec3f = vec3f(1.0);
    var pixel_color: vec3f = vec3f(0.0);
    for (var i: u32 = 0; i < sampling_parameters.num_bounces; i++) {
        var currentPayload = HitPayload();

        if TraceRay(nextRay, &currentPayload) {
            multiplier *= 0.5;
            var randomBounce: vec3f = normalize(rngNextVec3InUnitSphere(state));
            nextRay.origin = currentPayload.p + 0.0001 * currentPayload.n;
            if dot(randomBounce, currentPayload.n) < 0.0 {
                randomBounce *= -1.0;
            }
            nextRay.direction =  randomBounce;
        } else {
            let a: f32 = 0.5 * (primaryRay.direction.y + 1.0);
            pixel_color = multiplier * ((1.0 - a) * vec3f(1.0, 1.0, 1.0) + a * vec3f(0.5, 0.7, 1.0));
            break;
        }
    }

    return pixel_color;
}

fn TraceRay(ray: Ray, hit: ptr<function, HitPayload>) -> bool {
    var nearest_hit: f32 = 99999;
    let sphere_count = arrayLength(&spheres);
    var tempHitPayload = HitPayload();

    for (var i: u32 = 0; i < sphere_count; i++) {
        var newHitPayload = HitPayload();

        if hit(ray, i, 0.001, nearest_hit, &newHitPayload) {
            nearest_hit = newHitPayload.t;
            tempHitPayload = newHitPayload;
        }
    }

    if nearest_hit < 9999 {
        *hit = tempHitPayload;
        return true;
    }

    return false;
}

fn hit(ray: Ray, sphereIdx: u32, t_min: f32, t_nearest: f32, payload: ptr<function, HitPayload>) -> bool {
    let sphere: Sphere = spheres[sphereIdx];
    let a: f32 = dot(ray.direction, ray.direction);
    let b: f32 = dot(ray.direction, ray.origin - sphere.center);
    let c: f32 = dot(ray.origin - sphere.center, ray.origin - sphere.center) -
        sphere.radius * sphere.radius;
    let discrim: f32 = b * b - a * c;


    if (discrim >= 0) {
        var t: f32 = (-b - sqrt(discrim)) / a;
        if (t > t_min && t < t_nearest) {
            *payload = hitSphere(t, ray, sphere, sphereIdx, true);
            return true;
        }

        t = (-b + sqrt(discrim)) / a;
        if (t > t_min && t < t_nearest) {
            *payload = hitSphere(t, ray, sphere, sphereIdx, true);
            return true;
        }
    }
    return false;
}

fn hitSphere(t: f32, ray: Ray, sphere: Sphere, idx: u32, hit: bool) -> HitPayload {
    // make the hitPayload struct
    let p: vec3f = ray.origin + t * ray.direction;
    let n: vec3f = normalize(p - sphere.center);
    let color: vec3f = 0.5 * (n + vec3<f32>(1.0, 1.0, 1.0));

    return HitPayload(t, p, n , color, idx, hit);
}

fn getRay(pixel_00: vec3f, x: u32, y: u32, du: vec3f, dv: vec3f, state: ptr<function, u32>) -> Ray {
    let offset: vec3f = rngNextVec3InUnitDisk(state);

    var ray: Ray;
    ray.origin = camera.pos + offset.x * du + offset.y * dv;
    ray.direction = normalize(pixel_00 + f32(x) * du + f32(y) * dv - camera.pos);
    return ray;
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