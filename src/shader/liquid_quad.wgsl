const refractive_index: f32 = 1.77;
const frost_color: vec3<f32> = vec3<f32>(1.0);
const kernel_radius: i32 = 24;
const kernel_size: i32 = kernel_radius * 2 + 1;

struct FrameMetadata {
    resolution: vec2<u32>,
    top_left: vec2<i32>,
}
@group(0) @binding(0)
var<uniform> frame_metadata: FrameMetadata;

@group(0) @binding(1)
var<storage> quads: array<vec2<i32>>;

struct MetaFieldMetadata {
    offset: vec2<i32>,
    cell_size: u32,
}
@group(1) @binding(0)
var<uniform> metadata: MetaFieldMetadata;

@group(1) @binding(1)
var meta_field_texture: texture_2d<f32>;

@group(2) @binding(0)
var background_texture: texture_2d<f32>;

@group(2) @binding(1)
var background_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

override radius: f32;
override fade_dist: f32;
override height: f32;

override radius_sq: f32 = radius * radius;
override fade_dist_sq: f32 = fade_dist * fade_dist;
override height_sq: f32 = height * height;

@vertex
fn vert_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let pixel_pos = vec2<f32>(quads[instance_index * 4u + vertex_index]);
    let clip_pos = (pixel_pos / vec2<f32>(frame_metadata.resolution)) * 2.0 - 1.0;
    let final_pos = vec2<f32>(clip_pos.x, -clip_pos.y);

    return VertexOutput(
        vec4<f32>(final_pos, 0.0, 1.0),
    );
}

@fragment
fn frag_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let meta_field_coord = (frag_coord.xy + 0.5 - vec2<f32>(metadata.offset)) / f32(metadata.cell_size);


    let mag_tl = normalize_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(-1.0, -1.0)));
    let mag_tp = normalize_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(0.0, -1.0)));
    let mag_tr = normalize_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(1.0, -1.0)));
    let mag_lf = normalize_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(-1.0, 0.0)));
    let mag_rg = normalize_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(1.0, 0.0)));
    let mag_bl = normalize_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(-1.0, 1.0)));
    let mag_bm = normalize_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(0.0, 1.0)));
    let mag_br = normalize_meta_mag(load_meta_mag_bilinear(meta_field_coord + vec2<f32>(1.0, 1.0)));

    let grad = vec2<f32>(
        (mag_tr + 2.0 * mag_rg + mag_br) - (mag_tl + 2.0 * mag_lf + mag_bl),
        (mag_bl + 2.0 * mag_bm + mag_br) - (mag_tl + 2.0 * mag_tp + mag_tr),
    ) / (4.0 * 2.0 * f32(metadata.cell_size));
    let grad_mag = length(grad);

    let normal = normalize(vec3<f32>(-grad, 1.0));
    let view_dir = vec3<f32>(0.0, 0.0, -1.0);
    let refracted_dir = refract(view_dir, normal, 1.0 / refractive_index);

    let mag_height = normalize_meta_mag(load_meta_mag_bilinear(meta_field_coord));
    let offset_coord = frag_coord.xy + refracted_dir.xy * mag_height * f32(metadata.cell_size);
    let rgb = sample_background_blurred(offset_coord / vec2<f32>(frame_metadata.resolution));

    let frost_alpha = saturate(length(grad)) / 50.0 + 0.1;
    let frosted_rgb = mix(rgb, frost_color, frost_alpha);

    return vec4<f32>(frosted_rgb, 1.0);
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
    let inverted = invert_meta_mag(mag);
    let edge = radius - inverted;
    return select(
        rounded_plateau(edge),
        0.0,
        inverted >= radius,
    );
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

fn sample_background_blurred(uv: vec2<f32>) -> vec3<f32> {
    let texel_size = 1.0 / vec2<f32>(frame_metadata.resolution);
    let half = kernel_size / 2;
    let sigma = f32(kernel_radius) / 3.0;
    let inv_two_sigma2 = 0.5 / (sigma * sigma);

    var accum = vec3<f32>(0.0);
    var weight_sum = 0.0;

    for (var y: i32 = -half; y <= half; y = y + 1) {
        for (var x: i32 = -half; x <= half; x = x + 1) {
            let r2 = f32(x * x + y * y);
            let weight = exp(-r2 * inv_two_sigma2);
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            let sample_uv = uv + offset;
            let rgb = select(
                vec3<f32>(0.0),
                textureSample(background_texture, background_sampler, sample_uv).rgb,
                all(sample_uv >= vec2<f32>(0.0)) && all(sample_uv <= vec2<f32>(1.0)),
            );
            accum = accum + rgb * weight;
            weight_sum = weight_sum + weight;
        }
    }

    return accum / weight_sum;
}