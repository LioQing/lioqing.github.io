struct FrameMetadata {
    resolution: vec2<u32>,
    top_left: vec2<i32>,
}
@group(0) @binding(0)
var<uniform> frame_metadata: FrameMetadata;

struct MetaFieldMetadata {
    offset: vec2<i32>,
    cell_size: u32,
    fade_dist: u32,
}
@group(0) @binding(1)
var<uniform> metadata: MetaFieldMetadata;

@group(0) @binding(2)
var texture: texture_storage_2d<r32float, write>;

struct MetaShapesMetadata {
    ball_count: u32,
    line_count: u32,
    box_count: u32,
    padding: u32, // Why is padding not automatically inserted??
}
struct MetaShapes {
    metadata: MetaShapesMetadata,
    data: array<f32>,
}
@group(0) @binding(3)
var<storage> meta_shapes: MetaShapes;

override workgroup_size_x: u32;
override workgroup_size_y: u32;

@compute @workgroup_size(workgroup_size_x, workgroup_size_y)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id.xy >= frame_metadata.resolution) {
        return;
    }

    let cell_pos = vec2<i32>(id.xy) * i32(metadata.cell_size) + metadata.offset + frame_metadata.top_left;

    var value = 0.0;
    for (var i = 0u; i < meta_shapes.metadata.ball_count; i += 1u) {
        let ball_offset = i * 3u;
        let center = vec2<f32>(
            meta_shapes.data[ball_offset + 0u],
            meta_shapes.data[ball_offset + 1u],
        );
        let radius = meta_shapes.data[ball_offset + 2u];

        let disp = vec2<f32>(cell_pos) - center;
        value += implicit(length(disp), radius);
    }

    let line_data_start = meta_shapes.metadata.ball_count * 3u;
    for (var i = 0u; i < meta_shapes.metadata.line_count; i += 1u) {
        let line_offset = line_data_start + i * 5u;
        let start = vec2<f32>(
            meta_shapes.data[line_offset + 0u],
            meta_shapes.data[line_offset + 1u],
        );
        let end = vec2<f32>(
            meta_shapes.data[line_offset + 2u],
            meta_shapes.data[line_offset + 3u],
        );
        let radius = meta_shapes.data[line_offset + 4u];

        let line_vec = end - start;
        let line_len_sq = dot(line_vec, line_vec);
        let to_start = vec2<f32>(cell_pos) - start;
        let to_start_dot_line = dot(to_start, line_vec);
        let closest_point = select(
            select(
                start + to_start_dot_line / line_len_sq * line_vec,
                end,
                to_start_dot_line >= line_len_sq,
            ),
            start,
            to_start_dot_line <= 0.0,
        );
        let disp = vec2<f32>(cell_pos) - closest_point;
        value += implicit(length(disp), radius);
    }

    let box_data_start = line_data_start + meta_shapes.metadata.line_count * 5u;
    for (var i = 0u; i < meta_shapes.metadata.box_count; i += 1u) {
        let box_offset = box_data_start + i * 5u;
        let min = vec2<f32>(
            meta_shapes.data[box_offset + 0u],
            meta_shapes.data[box_offset + 1u],
        );
        let max = vec2<f32>(
            meta_shapes.data[box_offset + 2u],
            meta_shapes.data[box_offset + 3u],
        );
        let radius = meta_shapes.data[box_offset + 4u];

        let clamped = clamp(vec2<f32>(cell_pos), min, max);
        let disp = vec2<f32>(cell_pos) - clamped;
        value += select(
            implicit(length(disp), radius),
            1e8,
            all(clamped == vec2<f32>(cell_pos)),
        );
    }

    value = min(value, 1e8);

    textureStore(texture, id.xy, vec4<f32>(value, 0.0, 0.0, 0.0));
}

fn implicit(dist: f32, radius: f32) -> f32 {
    let fade_dist = f32(metadata.fade_dist);
    let implicit = radius * radius / (dist * dist);
    let fade_factor = saturate((dist - radius) / fade_dist);
    return implicit * (1.0 - fade_factor);
}