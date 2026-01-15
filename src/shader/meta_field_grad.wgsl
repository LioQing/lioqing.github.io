struct MetaFieldMetadata {
    offset: vec2<i32>,
    cell_size: u32,
}
@group(0) @binding(0)
var<uniform> metadata: MetaFieldMetadata;

@group(0) @binding(1)
var meta_field_texture: texture_2d<f32>;

override cell_size: u32;

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

    let mag_tl = max(0.0, load_meta_mag_bilinear(meta_field_coord + vec2<f32>(-1.0, -1.0)));
    let mag_tp = max(0.0, load_meta_mag_bilinear(meta_field_coord + vec2<f32>(0.0, -1.0)));
    let mag_tr = max(0.0, load_meta_mag_bilinear(meta_field_coord + vec2<f32>(1.0, -1.0)));
    let mag_lf = max(0.0, load_meta_mag_bilinear(meta_field_coord + vec2<f32>(-1.0, 0.0)));
    let mag_rg = max(0.0, load_meta_mag_bilinear(meta_field_coord + vec2<f32>(1.0, 0.0)));
    let mag_bl = max(0.0, load_meta_mag_bilinear(meta_field_coord + vec2<f32>(-1.0, 1.0)));
    let mag_bm = max(0.0, load_meta_mag_bilinear(meta_field_coord + vec2<f32>(0.0, 1.0)));
    let mag_br = max(0.0, load_meta_mag_bilinear(meta_field_coord + vec2<f32>(1.0, 1.0)));

    let grad = vec2<f32>(
        (mag_tr + 2.0 * mag_rg + mag_br) - (mag_tl + 2.0 * mag_lf + mag_bl),
        (mag_bl + 2.0 * mag_bm + mag_br) - (mag_tl + 2.0 * mag_tp + mag_tr),
    ) / f32(cell_size);
    let grad_mag = length(grad);

    let hue = atan2(grad.y, grad.x) / (2.0 * 3.14159265) + 0.5;
    let rgb = hsl_to_rgb(vec3<f32>(hue, 1.0, saturate(grad_mag)));

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

fn load_meta_mag_bilinear(coord: vec2<f32>) -> f32 {
    let base = vec2<i32>(floor(coord));
    let frac = coord - vec2<f32>(base);

    let v00 = load_meta_mag(base + vec2<i32>(0, 0));
    let v10 = load_meta_mag(base + vec2<i32>(1, 0));
    let v01 = load_meta_mag(base + vec2<i32>(0, 1));
    let v11 = load_meta_mag(base + vec2<i32>(1, 1));

    let vx0 = mix(v00, v10, frac.x);
    let vx1 = mix(v01, v11, frac.x);
    return mix(vx0, vx1, frac.y);
}