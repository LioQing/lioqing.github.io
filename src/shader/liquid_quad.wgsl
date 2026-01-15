const width: f32 = 2.0;
const half_width: f32 = width / 2.0;

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

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

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