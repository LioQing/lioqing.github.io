struct FrameMetadata {
    resolution: vec2<u32>,
    top_left: vec2<i32>,
}
@group(0) @binding(0)
var<uniform> frame_metadata: FrameMetadata;

@group(0) @binding(1)
var<uniform> grid_metadata: FrameMetadata;

@group(0) @binding(2)
var<uniform> target_pos: vec2<f32>;

@group(0) @binding(3)
var<uniform> delta_time: f32;

@group(0) @binding(4)
var<storage, read_write> pos: array<vec2<f32>>;

@group(0) @binding(5)
var<storage, read_write> vel: array<vec2<f32>>;

struct VertexOutput {
    @location(0) @interpolate(flat) radius: f32,
    @location(1) local_pixel_pos: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

override cell_size: u32;
override workgroup_size: u32;

fn clamp_length(v: vec2<f32>, max_len: f32) -> vec2<f32> {
    let len_sq = dot(v, v);
    let max_len_sq = max_len * max_len;
    if len_sq > max_len_sq {
        return v * (max_len / sqrt(len_sq));
    }
    return v;
}

@compute @workgroup_size(workgroup_size)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let grid_length = grid_metadata.resolution.x * grid_metadata.resolution.y;
    let is_outer = id.x >= grid_length;

    if id.x >= grid_length * 2u {
        return;
    }

    let cell_pos = vec2<i32>(
        i32(id.x % grid_length % grid_metadata.resolution.x),
        i32(id.x % grid_length / grid_metadata.resolution.x),
    );
    let center_pixel_pos = vec2<f32>(cell_pos * i32(cell_size) + grid_metadata.top_left);

    let index = id.x;
    var p = pos[index];
    var v = vel[index];

    let current_pixel_pos = center_pixel_pos + p;
    let disp_from_target = current_pixel_pos - target_pos;
    let dist_sq = dot(disp_from_target, disp_from_target);
    let dist = sqrt(dist_sq);

    let softening = 150.0;
    let influence_radius = 150.0 * select(1.0, 1.2, is_outer);
    let repel_strength = 15000.0 * select(1.0, 1.2, is_outer);
    let spring_k = 20.0 * select(1.0, 1.2, is_outer);
    let damping = 4.0 * select(1.0, 1.0 / 1.2, is_outer);

    let inv_r2 = 1.0 / (dist_sq + softening * softening);
    let dir = select(vec2<f32>(0.0), disp_from_target / max(dist, 1e-3), dist > 1e-3);
    let fade = 1.0 - smoothstep(0.0, influence_radius, dist);

    let a_repel = dir * (repel_strength * repel_strength * inv_r2) * fade;
    let a_spring = -spring_k * p;
    let a_damp = -damping * v;

    var a = a_repel + a_spring + a_damp;

    let max_accel = 250000.0;
    a = clamp_length(a, max_accel);

    v = v + a * delta_time;
    v = clamp_length(v, 6000.0);

    p = p + v * delta_time;

    pos[index] = p;
    vel[index] = v;
}