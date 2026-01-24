struct FrameMetadata {
    resolution: vec2<u32>,
    top_left: vec2<i32>,
}
@group(0) @binding(0)
var<uniform> frame_metadata: FrameMetadata;

struct MetaFieldMetadata {
    offset: vec2<i32>,
    cell_size: u32,
    padding: u32,
}
@group(0) @binding(1)
var<uniform> metadata: MetaFieldMetadata;

@group(0) @binding(2)
var texture: texture_storage_2d<rg32float, write>;

override base_radius: f32;
override fade_dist: f32;
override workgroup_size_x: u32;
override workgroup_size_y: u32;

struct MetaBall {
    center: vec2<f32>,
    radius: f32,
    hidden: u32,
}

struct MetaLine {
    start: vec2<f32>,
    end: vec2<f32>,
}

struct MetaBox {
    min: vec2<f32>,
    max: vec2<f32>,
    elevation: f32,
}

@group(0) @binding(3)
var<storage> balls: array<MetaBall>;

@group(0) @binding(4)
var<storage> boxes: array<MetaBox>;

@compute @workgroup_size(workgroup_size_x, workgroup_size_y)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id.xy >= frame_metadata.resolution) {
        return;
    }

    let cell_pos = vec2<i32>(id.xy) * i32(metadata.cell_size) + metadata.offset + frame_metadata.top_left;

    var meta_mag = 0.0;
    var elevation = 0.0;

    for (var i = 0u; i < arrayLength(&balls); i += 1u) {
        if balls[i].hidden == 1u {
            continue;
        }
        
        let ball = balls[i];
        let center = ball.center;
        let radius = ball.radius;

        let disp_from_center = vec2<f32>(cell_pos) - center;
        let dist = length(disp_from_center) - radius;
        meta_mag += implicit(max(0.0, dist), base_radius);
    }

    // for (var i = 0u; i < arrayLength(&lines); i += 1u) {
    //     let line = lines[i];
    //     let start = line.start;
    //     let end = line.end;

    //     let line_vec = end - start;
    //     let line_len_sq = dot(line_vec, line_vec);
    //     let to_start = vec2<f32>(cell_pos) - start;
    //     let to_start_dot_line = dot(to_start, line_vec);
    //     let closest_point = select(
    //         select(
    //             start + to_start_dot_line / line_len_sq * line_vec,
    //             end,
    //             to_start_dot_line >= line_len_sq,
    //         ),
    //         start,
    //         to_start_dot_line <= 0.0,
    //     );
    //     let disp = vec2<f32>(cell_pos) - closest_point;
    //     meta_mag += implicit(length(disp));
    // }

    for (var i = 0u; i < arrayLength(&boxes); i += 1u) {
        let box = boxes[i];
        let min = box.min;
        let max = box.max;

        let clamped = clamp(vec2<f32>(cell_pos), min, max);
        let disp = vec2<f32>(cell_pos) - clamped;
        let radius = base_radius + box.elevation;
        meta_mag += implicit(length(disp), radius);
        elevation += smooth_elevation(length(disp), box.elevation, radius);
    }

    meta_mag = min(meta_mag, 1e4);
    elevation = min(elevation, 1e4);

    textureStore(texture, id.xy, vec4<f32>(meta_mag, elevation, 0.0, 0.0));
}

fn implicit(dist: f32, radius: f32) -> f32 {
    if dist == 0.0 {
        return 1e4;
    }
    
    let implicit = radius * radius / (dist * dist);
    let fade_factor = min((dist - radius) / fade_dist, 1.0);
    return min(implicit * (1.0 - fade_factor), 1e4);
}

fn smooth_elevation(dist: f32, elevation: f32, radius: f32) -> f32 {
    return select(
        elevation,
        select(
            elevation * (1.0 - (dist - radius) / fade_dist),
            0.0,
            dist >= radius + fade_dist,
        ),
        dist > radius,
    );
}