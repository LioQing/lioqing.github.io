struct MetaFieldMetadata {
    offset: vec2<i32>,
    cell_size: u32,
}
@group(0) @binding(0)
var<uniform> metadata: MetaFieldMetadata;

@group(0) @binding(1)
var meta_field_texture: texture_2d<f32>;

override radius: f32;
override fade_dist: f32;
override height: f32;

override radius_sq: f32 = radius * radius;
override fade_dist_sq: f32 = fade_dist * fade_dist;
override height_sq: f32 = height * height;

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
    let mag = normalize_meta_mag(load_meta_mag_bilinear(meta_field_coord));
    let rgb = select(
        vec3<f32>(mag + 1.0),
        hsl_to_rgb(vec3<f32>(mag / height, 1.0, 0.25)),
        mag > 0.0,
    );

    return vec4<f32>(rgb, 1.0);
}

fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
   let c = vec3<f32>(fract(hsl.x), clamp(vec2<f32>(hsl.y, hsl.z), vec2<f32>(0.0), vec2<f32>(1.0)));
   let rgb = clamp(abs((c.x * 6.0 + vec3<f32>(0.0, 4.0, 2.0)) % 6.0 - 3.0) - 1.0, vec3<f32>(0.0), vec3<f32>(1.0));
   return c.z + c.y * (rgb - 0.5) * (1.0 - abs(2.0 * c.z - 1.0));
}

fn invert_meta_mag(mag: f32) -> f32 {
    return (
        -radius_sq + radius * sqrt(
            radius_sq
                + 4.0 * radius * fade_dist * mag
                + 4.0 * fade_dist_sq * mag
        )
    ) / (2.0 * fade_dist * mag);
}

fn rounded_plateau(x: f32) -> f32 {
    let circle_x = height - clamp(x, 0.0, height);
    let segment = sqrt(height_sq - circle_x * circle_x);
    return segment * (1.0 - x / height) + x;
}

fn normalize_meta_mag(mag: f32) -> f32 {
    if mag < 1.0 {
        return -1e4;
    }

    let inverted = invert_meta_mag(mag);
    let edge = radius - inverted;
    return rounded_plateau(edge);
}

fn load_meta_mag(coord: vec2<i32>) -> f32 {
    let texture_dim = textureDimensions(meta_field_texture);
    let mag = textureLoad(
        meta_field_texture,
        clamp(coord, vec2<i32>(0), vec2<i32>(texture_dim) - vec2<i32>(1)),
        0,
    ).x;

    return mag;
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