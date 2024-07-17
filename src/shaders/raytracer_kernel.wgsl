struct Sphere {
    center: vec3<f32>,
    radius: f32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

struct SceneData {
    cameraPos: vec3<f32>,
    cameraForwards: vec3<f32>,
    cameraRight: vec3<f32>,
    cameraUp: vec3<f32>,
    sphereCount: f32,
}

@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> scene: SceneData;
@compute @workgroup_size(1,1,1)
fn main(@builtin(global_invocation_id) id: vec3u) {

    let screen_size: vec2<u32> = textureDimensions(color_buffer);
    let screen_pos = id.xy;

    let horiz_dx: f32 = (f32(screen_pos.x) - f32(screen_size.x) / 2) / f32(screen_size.x);
    let vert_dy: f32 = (f32(screen_pos.y) - f32(screen_size.y) / 2) / f32(screen_size.x);

    let forwards: vec3<f32> = scene.cameraForwards;
    let right: vec3<f32> = scene.cameraRight;
    let up: vec3<f32> = scene.cameraUp;

    var mySphere: Sphere;
    mySphere.center = vec3<f32>(3.0, 0.0, 0.0);
    mySphere.radius = 1.0;

    var myRay: Ray;
    myRay.origin = scene.cameraPos;
    myRay.direction = normalize(forwards + horiz_dx * right + vert_dy * up);

    var pixel_color: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0);

    if (hit(myRay, mySphere)) {
        pixel_color = vec3<f32>(0.5, 1.0, 0.75);
    }

    textureStore(color_buffer, screen_pos, vec4<f32>(pixel_color, 1.0));
}

fn hit(ray: Ray, sphere: Sphere) -> bool {
    let a: f32 = dot(ray.direction, ray.direction);
    let b: f32 = dot(ray.direction, ray.origin - sphere.center);
    let c: f32 = dot(ray.origin - sphere.center, ray.origin - sphere.center) -
        sphere.radius * sphere.radius;
    let discrim: f32 = b * b - a * c;

    return discrim >= 0;
}