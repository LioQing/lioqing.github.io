const refractive_index: f32 = 1.77;
const quad_height: f32 = 18.0;
const elevation_scale: f32 = 18.0;
const frost_rgb: vec3<f32> = vec3<f32>(0.05);
const frost_strength: f32 = 0.1;
const frost_absorption: f32 = 0.01;

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

@group(2) @binding(2)
var<uniform> background_color: vec3<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

override base_radius: f32;
override fade_dist: f32;
override base_height: f32;

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

    let mag_tl = load_shaped_meta_mag(meta_field_coord + vec2<f32>(-1.0, -1.0));
    let mag_tp = load_shaped_meta_mag(meta_field_coord + vec2<f32>(0.0, -1.0));
    let mag_tr = load_shaped_meta_mag(meta_field_coord + vec2<f32>(1.0, -1.0));
    let mag_lf = load_shaped_meta_mag(meta_field_coord + vec2<f32>(-1.0, 0.0));
    let mag_rg = load_shaped_meta_mag(meta_field_coord + vec2<f32>(1.0, 0.0));
    let mag_bl = load_shaped_meta_mag(meta_field_coord + vec2<f32>(-1.0, 1.0));
    let mag_bm = load_shaped_meta_mag(meta_field_coord + vec2<f32>(0.0, 1.0));
    let mag_br = load_shaped_meta_mag(meta_field_coord + vec2<f32>(1.0, 1.0));

    let grad_dir_unnormalized = vec2<f32>(
        (mag_tr.x + 2.0 * mag_rg.x + mag_br.x) - (mag_tl.x + 2.0 * mag_lf.x + mag_bl.x),
        (mag_bl.x + 2.0 * mag_bm.x + mag_br.x) - (mag_tl.x + 2.0 * mag_tp.x + mag_tr.x),
    ) / (4.0 * 2.0 * f32(metadata.cell_size));
    let grad_dir = select(
        vec2<f32>(0.0),
        normalize(grad_dir_unnormalized),
        grad_dir_unnormalized != vec2<f32>(0.0),
    );

    let meta_mag = load_meta_mag_bilinear(meta_field_coord);
    let grad_mag = differentiate_meta_mag(meta_mag.x, base_radius + meta_mag.y, base_height + meta_mag.y);

    let grad = grad_dir * grad_mag;

    let screen_depth = f32(frame_metadata.resolution.y) * 2.0;
    let normal = normalize(vec3<f32>(-grad, 1.0));
    let screen_center = vec2<f32>(frame_metadata.resolution) * 0.5;
    let mag_height = shape_meta_mag(meta_mag.x, base_radius + meta_mag.y, base_height + meta_mag.y);
    let total_height = mag_height + quad_height + meta_mag.y * elevation_scale;
    let view_dir = normalize(vec3<f32>(frag_coord.xy - screen_center, -screen_depth));

    let refracted_dir = refract(view_dir, normal, 1.0 / refractive_index);
    let refracted_rgb = cast_ray_at_background(refracted_dir, frag_coord, total_height);

    let reflected_cos_theta = saturate(dot(-view_dir, normal));
    let reflected_strength = pow(1.0 - reflected_cos_theta, 4.0);

    let reflected_dir = reflect(view_dir, normal);
    let reflected_background_rgb = cast_ray_at_background(reflected_dir, frag_coord, total_height);

    const light_dir_tl: vec3<f32> = normalize(vec3<f32>(0.2, 0.4, 1.0));
    const light_dir_br: vec3<f32> = normalize(vec3<f32>(-0.2, -0.4, 1.0));
    const light_rgb: vec3<f32> = vec3<f32>(1.0);
    const light_intensity_tl: f32 = 0.8;
    const light_intensity_br: f32 = 0.5;

    let mirror_dir_tl = reflect(light_dir_tl, normal);
    let spec_angle_tl = saturate(dot(mirror_dir_tl, -view_dir));
    let spec_tl = pow(spec_angle_tl, 64.0);

    let mirror_dir_br = reflect(light_dir_br, normal);
    let spec_angle_br = saturate(dot(mirror_dir_br, -view_dir));
    let spec_br = pow(spec_angle_br, 64.0);

    let reflected_light_rgb = light_rgb * (
        spec_tl * light_intensity_tl
            + spec_br * light_intensity_br
    );

    let reflected_rgb = reflected_background_rgb + reflected_light_rgb;

    let absorbed_rgb = mix(frost_rgb, refracted_rgb, exp(-frost_absorption * total_height));
    let ray_rgb = mix(absorbed_rgb, reflected_rgb, reflected_strength);
    let frosted_rgb = mix(ray_rgb, frost_rgb, frost_strength);

    let final_rgb = frosted_rgb;

    return vec4<f32>(final_rgb, 1.0);
}

fn differentiate_rounded_plateau(x: f32, height: f32) -> f32 {
    if x <= 0.0 || x >= height {
        return 0.0;
    }
    
    let height_sq = height * height;
    let height_2_sub_x = height * 2.0 - x;
    let height_sqrt_2_sub_x = height * sqrt(height_2_sub_x * x);
    return (
        height_sq
            + 2.0 * x * x
            - 4.0 * x * height
            + height_sqrt_2_sub_x
    ) / height_sqrt_2_sub_x;
}

fn differentiate_meta_mag(mag: f32, radius: f32, height: f32) -> f32 {
    let edge = edge_meta_mag(mag, radius);
    let deriv = differentiate_rounded_plateau(edge, height);
    return deriv;
}

fn edge_meta_mag(mag: f32, radius: f32) -> f32 {
    let inverted = invert_meta_mag(mag, radius);
    let edge = radius - inverted;
    return edge;
}

fn invert_meta_mag(mag: f32, radius: f32) -> f32 {
    if mag == 0.0 {
        return 0.0;
    }
    
    let radius_sq = radius * radius;
    return (
        -radius_sq + radius * sqrt(
            radius_sq
                + 4.0 * radius * fade_dist * mag
                + 4.0 * fade_dist * fade_dist * mag
        )
    ) / (2.0 * fade_dist * mag);
}

fn rounded_plateau(x: f32, height: f32) -> f32 {
    let circle_x = height - clamp(x, 0.0, height);
    let segment = sqrt(height * height - circle_x * circle_x);
    return segment * (1.0 - x / height) + x;
}

fn shape_meta_mag(mag: f32, radius: f32, height: f32) -> f32 {
    if mag < 1.0 {
        return 0.0;
    }

    let edge = edge_meta_mag(mag, radius);
    let plateau = rounded_plateau(edge, height);
    return plateau;
}

fn load_meta_mag(coord: vec2<i32>) -> vec2<f32> {
    let texture_dim = textureDimensions(meta_field_texture);
    let mag = textureLoad(
        meta_field_texture,
        clamp(coord, vec2<i32>(0), vec2<i32>(texture_dim) - vec2<i32>(1)),
        0,
    ).xy;

    return mag;
}

fn load_meta_mag_bilinear(coord: vec2<f32>) -> vec2<f32> {
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

fn load_shaped_meta_mag(coord: vec2<f32>) -> vec2<f32> {
    let mag = load_meta_mag_bilinear(coord);
    let mag_shaped = shape_meta_mag(mag.x, base_radius + mag.y, base_height + mag.y);
    return vec2<f32>(mag_shaped, mag.y);
}

fn sample_background(uv: vec2<f32>) -> vec3<f32> {
    let sample_uv = saturate(uv);
    return textureSample(background_texture, background_sampler, sample_uv).rgb;
}

fn cast_ray_at_background(ray: vec3<f32>, frag_coord: vec4<f32>, height: f32) -> vec3<f32> {    
    let refracted_coord = frag_coord.xy + ray.xy * (height / -ray.z);
    let background_rgb = sample_background(refracted_coord / vec2<f32>(frame_metadata.resolution));
    return select(background_color, background_rgb, ray.z < 0.0);
}