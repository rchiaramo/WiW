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
    color: vec3<f32>,
    hit: bool,
}

struct SceneData {
    cameraPos: vec3<f32>,
    cameraForwards: vec3<f32>,
    cameraRight: vec3<f32>,
    cameraUp: vec3<f32>,
}

@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> scene: SceneData;
@group(0) @binding(2) var<storage, read> spheres: array<Sphere>;
@compute @workgroup_size(1,1,1)
fn main(@builtin(global_invocation_id) id: vec3u) {

    let image_size: vec2<u32> = textureDimensions(color_buffer);
    let screen_pos = id.xy;

    let viewport_height: f32 = 2.0;
    let viewport_width: f32 = viewport_height * (f32(image_size.x) / f32(image_size.y));
    let viewport_u: vec3f = vec3f(viewport_width, 0.0, 0.0);
    let viewport_v: vec3f = vec3f(0.0, -viewport_height, 0.0);
    let du: vec3f = vec3f(viewport_width / f32(image_size.x), 0.0, 0.0);
    let dv: vec3f = vec3f(0.0, -viewport_height / f32(image_size.y), 0.0);
    let upper_left: vec3f = scene.cameraPos - vec3f(0.0, 0.0, 1.0) - viewport_u / 2.0 - viewport_v / 2.0;

    let horiz_dx: f32 = 2.0 * (f32(screen_pos.x) / f32(image_size.x)) - 1.0;
    let vert_dy: f32 = 2.0 * (1.0 - f32(screen_pos.y) / f32(image_size.y)) - 1.0;

    let forwards: vec3<f32> = scene.cameraForwards;
    let right: vec3<f32> = scene.cameraRight;
    let up: vec3<f32> = scene.cameraUp;

    var myRay: Ray;
    myRay.origin = scene.cameraPos;
    myRay.direction = normalize(upper_left + f32(id.x) * du + f32(id.y) * dv - scene.cameraPos);

    let pixel_color: vec3<f32> = rayColor(myRay);
//    let pixel_color: vec3<f32> = vec3<f32>(f32(screen_pos.x) / f32(screen_size.x), f32(screen_pos.y) / f32(screen_size.y), 0.0);

    textureStore(color_buffer, screen_pos, vec4<f32>(pixel_color, 1.0));
}

fn rayColor(ray: Ray) -> vec3<f32> {
    var pixel_color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
    var nearest_hit: f32 = 99999;
    var hit_something: bool = false;

    var renderState: HitPayload;
    let sphere_count = arrayLength(&spheres);
    for (var i: u32 = 0; i < sphere_count; i++) {
        var newHitPayload: HitPayload = hit(ray, spheres[i], 0.001, nearest_hit, renderState);

        if (newHitPayload.hit) {
            nearest_hit = newHitPayload.t;
            renderState = newHitPayload;
            hit_something = true;
        }
    }

    if (hit_something) {
        pixel_color = renderState.color;
    }
    return pixel_color;
}

fn hit(ray: Ray, sphere: Sphere, t_min: f32, t_nearest: f32, oldRenderState: HitPayload) -> HitPayload {
    let a: f32 = dot(ray.direction, ray.direction);
    let b: f32 = dot(ray.direction, ray.origin - sphere.center);
    let c: f32 = dot(ray.origin - sphere.center, ray.origin - sphere.center) -
        sphere.radius * sphere.radius;
    let discrim: f32 = b * b - a * c;

    var renderState: HitPayload;
    renderState.color = oldRenderState.color;

    if (discrim >= 0) {
        let t: f32 = (-b - sqrt(discrim)) / a;
        if (t > t_min && t < t_nearest) {
            renderState.hit = true;
            renderState.t = t;
            let p: vec3<f32> = ray.origin + t * ray.direction;
            let n: vec3<f32> = normalize(p - sphere.center);
            renderState.color = 0.5 * (n + vec3<f32>(1.0, 1.0, 1.0));
            return renderState;
        }
    }
    renderState.hit = false;
    return renderState;
}