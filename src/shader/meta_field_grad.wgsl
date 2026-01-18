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

    let mag_tl = shape_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(-1.0, -1.0)));
    let mag_tp = shape_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(0.0, -1.0)));
    let mag_tr = shape_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(1.0, -1.0)));
    let mag_lf = shape_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(-1.0, 0.0)));
    let mag_rg = shape_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(1.0, 0.0)));
    let mag_bl = shape_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(-1.0, 1.0)));
    let mag_bm = shape_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(0.0, 1.0)));
    let mag_br = shape_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(1.0, 1.0)));

    let grad_dir_unnormalized = vec2<f32>(
        (mag_tr + 2.0 * mag_rg + mag_br) - (mag_tl + 2.0 * mag_lf + mag_bl),
        (mag_bl + 2.0 * mag_bm + mag_br) - (mag_tl + 2.0 * mag_tp + mag_tr),
    );
    let grad_dir = select(
        vec2<f32>(0.0),
        normalize(grad_dir_unnormalized),
        grad_dir_unnormalized != vec2<f32>(0.0),
    );

    // TODO: Right now only the magnitude of the gradient is computed analytically,
    // maybe at some point we can output vec2 from `meta_field_processs` to the texture
    // so that we can differentiate both direction and magnitude numerically.
    let grad_mag = differentiate_meta_mag(load_meta_mag_bilinear(meta_field_coord));

    let grad = grad_dir * grad_mag;

    let normal = normalize(vec3<f32>(-grad, 1.0));

    let hue = atan2(normal.y, normal.x) / (2.0 * 3.14159265);
    let rgb = hsl_to_rgb(vec3<f32>(normal.z, 1.0, 0.5));

    return vec4<f32>(rgb, 1.0);
}

fn hsl_to_rgb(hsl: vec3<f32>) -> vec3<f32> {
   let c = vec3<f32>(fract(hsl.x), clamp(vec2<f32>(hsl.y, hsl.z), vec2<f32>(0.0), vec2<f32>(1.0)));
   let rgb = clamp(abs((c.x * 6.0 + vec3<f32>(0.0, 4.0, 2.0)) % 6.0 - 3.0) - 1.0, vec3<f32>(0.0), vec3<f32>(1.0));
   return c.z + c.y * (rgb - 0.5) * (1.0 - abs(2.0 * c.z - 1.0));
}

fn differentiate_rounded_plateau(x: f32) -> f32 {
    if x <= 0.0 || x >= height {
        return 0.0;
    }
    
    let height_2_sub_x = height * 2.0 - x;
    let height_sqrt_2_sub_x = height * sqrt(height_2_sub_x * x);
    return (
        height_sq
            + 2.0 * x * x
            - 4.0 * x * height
            + height_sqrt_2_sub_x
    ) / height_sqrt_2_sub_x;
}

fn differentiate_meta_mag(mag: f32) -> f32 {
    let edge = edge_meta_mag(mag);
    let deriv = differentiate_rounded_plateau(edge);
    return deriv;
}

fn edge_meta_mag(mag: f32) -> f32 {
    let inverted = invert_meta_mag(mag);
    let edge = radius - inverted;
    return edge;
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

fn shape_meta_mag(mag: f32) -> f32 {
    if mag < 1.0 {
        return 0.0;
    }

    let edge = edge_meta_mag(mag);
    let plateau = rounded_plateau(edge);
    return plateau;
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