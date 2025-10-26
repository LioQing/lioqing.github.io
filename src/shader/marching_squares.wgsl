struct MetaFieldMetadata {
    offset: vec2<i32>,
    cell_size: u32,
}
@group(0) @binding(0)
var<uniform> metadata: MetaFieldMetadata;

@group(0) @binding(1)
var texture: texture_storage_2d<r32float, read>;

struct DrawIndirectArgs {
    vertex_count: u32,
    instance_count: atomic<u32>,
    first_vertex: u32,
    first_instance: u32,
}
@group(0) @binding(2)
var<storage, read_write> line_segment_indirect_args: DrawIndirectArgs;

@group(0) @binding(3)
var<storage, read_write> line_segments: array<vec4<i32>>;

override workgroup_size: u32;

@compute @workgroup_size(workgroup_size)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let texture_dim = textureDimensions(texture);
    let coord = vec2<u32>(
        id.x % texture_dim.x,
        id.x / texture_dim.x,
    );

    let top_left_coord = coord;
    let top_right_coord = coord + vec2<u32>(1, 0);
    let bottom_right_coord = coord + vec2<u32>(1, 1);
    let bottom_left_coord = coord + vec2<u32>(0, 1);

    let top_left = textureLoad(texture, top_left_coord).x;
    let top_right = textureLoad(texture, top_right_coord).x;
    let bottom_right = textureLoad(texture, bottom_right_coord).x;
    let bottom_left = textureLoad(texture, bottom_left_coord).x;

    let patt = (
        (u32(top_left > 1.0) << 3u) |
        (u32(top_right > 1.0) << 2u) |
        (u32(bottom_right > 1.0) << 1u) |
        (u32(bottom_left > 1.0) << 0u)
    );

    if patt == 0u || patt == 15u {
        return;
    }

    let top_left_pos = vec2<f32>(top_left_coord);
    let top_right_pos = vec2<f32>(top_right_coord);
    let bottom_right_pos = vec2<f32>(bottom_right_coord);
    let bottom_left_pos = vec2<f32>(bottom_left_coord);

    var index = 0u;
    var new_line_segment_indices: array<u32, 2>;
    var new_line_segments: array<vec2<f32>, 4>;

    // Top
    if patt >= 4u && patt <= 11u {
        let p = lerp(top_left_pos, top_right_pos, top_left, top_right);
        new_line_segments[index] = p;

        if index == 0u {
            new_line_segment_indices[0] = atomicAdd(&line_segment_indirect_args.instance_count, 1u);
        }

        index += 1;
    }

    // Left
    if ((patt >> 3u) & 1u) != (patt & 1u) {
        let p = lerp(top_left_pos, bottom_left_pos, top_left, bottom_left);
        new_line_segments[index] = p;

        if index == 0u {
            new_line_segment_indices[0] = atomicAdd(&line_segment_indirect_args.instance_count, 1u);
        }

        index += 1;
    }

    // Bottom
    if (patt & 1u) != ((patt >> 1u) & 1u) {
        let p = lerp(bottom_left_pos, bottom_right_pos, bottom_left, bottom_right);
        new_line_segments[index] = p;

        if index == 0u {
            new_line_segment_indices[0] = atomicAdd(&line_segment_indirect_args.instance_count, 1u);
        } else if index == 2u {
            new_line_segment_indices[1] = atomicAdd(&line_segment_indirect_args.instance_count, 1u);
        }

        index += 1;
    }

    // Right
    if ((patt >> 1u) & 1u) != ((patt >> 2u) & 1u) {
        let p = lerp(top_right_pos, bottom_right_pos, top_right, bottom_right);
        new_line_segments[index] = p;

        index += 1;
    }

    line_segments[new_line_segment_indices[0]] = vec4<i32>(
        vec2<i32>(new_line_segments[0] * f32(metadata.cell_size)) + metadata.offset,
        vec2<i32>(new_line_segments[1] * f32(metadata.cell_size)) + metadata.offset,
    );

    if index == 4u {
        line_segments[new_line_segment_indices[1]] = vec4<i32>(
            vec2<i32>(new_line_segments[2] * f32(metadata.cell_size)) + metadata.offset,
            vec2<i32>(new_line_segments[3] * f32(metadata.cell_size)) + metadata.offset,
        );
    }
}

fn lerp(p1: vec2<f32>, p2: vec2<f32>, v1: f32, v2: f32) -> vec2<f32> {
    let t = (1.0 - v1) / (v2 - v1);
    return p1 + t * (p2 - p1);
}