struct MetaFieldMetadata {
    offset: vec2<i32>,
    cell_size: u32,
}
@group(0) @binding(0)
var<uniform> metadata: MetaFieldMetadata;

@group(0) @binding(1)
var meta_field_texture: texture_2d<f32>;

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
    let meta_field_coord = vec2<i32>((frag_coord.xy + 0.5 - vec2<f32>(metadata.offset)) / f32(metadata.cell_size));
    let mag = load_meta_mag(meta_field_coord);
    let rgb = select(
        vec3<f32>(0.0),
        hsl_to_rgb(vec3<f32>(mag, 1.0, 0.5)),
        mag >= 0.0,
    );

    return vec4<f32>(rgb, 1.0);
}

fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
   let c = vec3<f32>(fract(hsl.x), clamp(vec2<f32>(hsl.y, hsl.z), vec2<f32>(0.0), vec2<f32>(1.0)));
   let rgb = clamp(abs((c.x * 6.0 + vec3<f32>(0.0, 4.0, 2.0)) % 6.0 - 3.0) - 1.0, vec3<f32>(0.0), vec3<f32>(1.0));
   return c.z + c.y * (rgb - 0.5) * (1.0 - abs(2.0 * c.z - 1.0));
}

fn load_meta_mag(coord: vec2<i32>) -> f32 {
    let texture_dim = textureDimensions(meta_field_texture);
    let texture_val = textureLoad(meta_field_texture, coord, 0).x;
    let offset_val = select(
        -1.0,
        texture_val - 1.0,
        all(coord >= vec2<i32>(0)) && all(coord < vec2<i32>(texture_dim)),
    );
    return 1.0 - exp(-offset_val);
}