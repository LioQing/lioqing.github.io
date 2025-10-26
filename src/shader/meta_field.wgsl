struct MetaFieldMetadata {
    offset: vec2<i32>,
    cell_size: u32,
}
@group(0) @binding(0)
var<uniform> metadata: MetaFieldMetadata;

@group(0) @binding(1)
var meta_field_texture: texture_2d<f32>;

@group(0) @binding(2)
var meta_field_sampler: sampler;

@vertex
fn vert_main(@builtin(vertex_index) vert_index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(
        f32((vert_index << 1u) & 2u),
        f32(vert_index & 2u),
    );
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn frag_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let meta_field_coord = (frag_coord.xy + 0.5 - vec2<f32>(metadata.offset)) / f32(metadata.cell_size);
    let meta_field_uv = (meta_field_coord + 0.5) / vec2<f32>(textureDimensions(meta_field_texture));
    let magnitude = textureSample(meta_field_texture, meta_field_sampler, meta_field_uv).x;
    let value = 1.0 - exp(-magnitude);

    if magnitude > 1.0 {
        return vec4<f32>(0.0, value, 0.0, 1.0);
    }

    return vec4<f32>(vec3<f32>(value), 1.0);
}