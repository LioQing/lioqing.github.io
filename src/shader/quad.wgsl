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
    return vec4<f32>(0.8);
}