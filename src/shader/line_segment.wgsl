const width: f32 = 2.0;
const half_width: f32 = width / 2.0;

struct FrameMetadata {
    resolution: vec2<u32>,
    top_left: vec2<i32>,
}
@group(0) @binding(0)
var<uniform> frame_metadata: FrameMetadata;

@group(0) @binding(1)
var<storage> line_segments: array<vec4<i32>>;

struct VertexOutput {
    @location(0) @interpolate(flat) segment: vec4<i32>,
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vert_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let segment = line_segments[instance_index];

    let start = vec2<f32>(segment.xy);
    let end = vec2<f32>(segment.zw);
    
    let dir = normalize(end - start) * half_width;
    let perp = vec2<f32>(-dir.y, dir.x);

    let positions = array<vec2<f32>, 4>(
        start - perp - dir, // 0: bottom-left of start
        start + perp - dir, // 1: top-left of start
        end - perp + dir,   // 2: bottom-right of end
        end + perp + dir,   // 3: top-right of end
    );
    
    let pixel_pos = positions[vertex_index];
    
    let is_start = vertex_index <= 2u || vertex_index == 3u;
    let segment_end = select(start, end, is_start);
    let segment_tip = select(start - dir, end + dir, is_start);

    let clip_pos = (pixel_pos / vec2<f32>(frame_metadata.resolution)) * 2.0 - 1.0;
    let final_pos = vec2<f32>(clip_pos.x, -clip_pos.y);
    
    return VertexOutput(
        segment,
        vec4<f32>(final_pos, 0.0, 1.0),
    );
}

@fragment
fn frag_main(
    @location(0) @interpolate(flat) segment: vec4<i32>,
    @builtin(position) frag_coord: vec4<f32>
) -> @location(0) vec4<f32> {
    let start = vec2<f32>(segment.xy);
    let end = vec2<f32>(segment.zw);

    let at_end_side = dot(end - start, frag_coord.xy - start) > 0.0;
    let at_start_side = dot(start - end, frag_coord.xy - end) > 0.0;
    if at_end_side && at_start_side {
        return vec4<f32>(0.6);
    } else if at_end_side {
        if length(frag_coord.xy - end) <= half_width {
            return vec4<f32>(0.6);
        }
    } else if at_start_side {
        if length(frag_coord.xy - start) <= half_width {
            return vec4<f32>(0.6);
        }
    }
    
    discard;
    return vec4<f32>(0.0);
}