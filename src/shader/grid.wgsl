const width: f32 = 1.0;
const inner_radius: f32 = 1.0;
const outer_radius: f32 = 4.0;

struct FrameMetadata {
    resolution: vec2<u32>,
    top_left: vec2<i32>,
}
@group(0) @binding(0)
var<uniform> frame_metadata: FrameMetadata;

@group(0) @binding(1)
var<uniform> grid_metadata: FrameMetadata;

@group(0) @binding(2)
var<storage, read> state: array<vec2<f32>>;

struct VertexOutput {
    @location(0) @interpolate(flat) is_outer: u32,
    @location(1) @interpolate(flat) intensity: f32,
    @location(2) local_pixel_pos: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

override cell_size: u32;

@vertex
fn vert_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> VertexOutput {
    let grid_length = grid_metadata.resolution.x * grid_metadata.resolution.y;
    let is_outer = instance_index >= grid_length;
    let radius = select(inner_radius, outer_radius, is_outer);

    let cell_pos = vec2<i32>(
        i32(instance_index % grid_length % grid_metadata.resolution.x),
        i32(instance_index % grid_length / grid_metadata.resolution.x),
    );
    let center_pixel_pos = vec2<f32>(cell_pos * i32(cell_size) + grid_metadata.top_left);
    let target_pixel_pos = center_pixel_pos + state[instance_index];

    let pos_mag = length(state[instance_index]);
    let intensity = saturate(pos_mag / (1 + pos_mag) + select(0.5, 0.0, is_outer));

    let positions = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0), // 0: bottom-left
        vec2<f32>(-1.0,  1.0), // 1: top-left
        vec2<f32>( 1.0, -1.0), // 2: bottom-right
        vec2<f32>( 1.0,  1.0), // 3: top-right
    );

    let local_pixel_pos = positions[vertex_index] * radius;
    let pixel_pos = target_pixel_pos + local_pixel_pos;
    let clip_pos = (pixel_pos / vec2<f32>(frame_metadata.resolution)) * 2.0 - 1.0;
    let final_pos = vec2<f32>(clip_pos.x, -clip_pos.y);

    return VertexOutput(
        u32(is_outer),
        intensity,
        local_pixel_pos,
        vec4<f32>(final_pos, 0.0, 1.0),
    );
}

@fragment
fn frag_main(
    @location(0) @interpolate(flat) is_outer: u32,
    @location(1) @interpolate(flat) intensity: f32,
    @location(2) local_pixel_pos: vec2<f32>,
    @builtin(position) frag_coord: vec4<f32>
) -> @location(0) vec4<f32> {
    let radius = select(inner_radius, outer_radius, is_outer == 1u);
    let length = length(local_pixel_pos);

    if length > radius {
        discard;
    }

    if length <= radius - width {
        discard;
    }

    return vec4<f32>(vec3<f32>(0.0), intensity);
}