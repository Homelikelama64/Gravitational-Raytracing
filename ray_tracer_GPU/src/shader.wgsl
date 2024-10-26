@group(0)
@binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(16, 16)
fn trace_ray(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let size = textureDimensions(output_texture);
    let coords = global_id.xy;

    if coords.x >= size.x || coords.y >= size.y {
        return;
    }

    let color = vec3<f32>(1.0, 0.0, 0.0);
    textureStore(output_texture, coords, vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0));
}
