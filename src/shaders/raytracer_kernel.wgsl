@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(1,1,1)
fn main(@builtin(global_invocation_id) id: vec3u) {

    let screen_pos = id.xy;

    var pixel_color: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0);

    textureStore(color_buffer, screen_pos, vec4<f32>(pixel_color, 1.0));
}