// TODO: merge this into the rasterization shader

struct MetaFieldMetadata {
    offset: vec2<i32>,
    cell_size: u32,
}
@group(0) @binding(0)
var<uniform> metadata: MetaFieldMetadata;

@group(0) @binding(1)
var texture: texture_storage_2d<rg32float, read>;

struct DrawIndirectArgs {
    vertex_count: u32,
    instance_count: atomic<u32>,
    first_vertex: u32,
    first_instance: u32,
}
@group(0) @binding(2)
var<storage, read_write> quad_indirect_args: DrawIndirectArgs;

@group(0) @binding(3)
var<storage, read_write> quads: array<vec2<i32>>;

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
        (u32(top_left >= 1.0) << 3u) |
        (u32(top_right >= 1.0) << 2u) |
        (u32(bottom_right >= 1.0) << 1u) |
        (u32(bottom_left >= 1.0) << 0u)
    );

    let top_left_pos = vec2<f32>(top_left_coord);
    let top_right_pos = vec2<f32>(top_right_coord);
    let bottom_right_pos = vec2<f32>(bottom_right_coord);
    let bottom_left_pos = vec2<f32>(bottom_left_coord);

    let top_pos = lerp(top_left_pos, top_right_pos, top_left, top_right);
    let right_pos = lerp(top_right_pos, bottom_right_pos, top_right, bottom_right);
    let bottom_pos = lerp(bottom_left_pos, bottom_right_pos, bottom_left, bottom_right);
    let left_pos = lerp(top_left_pos, bottom_left_pos, top_left, bottom_left);

    switch patt {
        case 0u, default: {
        }

        // 1 point
        case 1u: {
            push_quad(bottom_pos, left_pos, bottom_left_pos, bottom_left_pos);
        }
        case 2u: {
            push_quad(right_pos, bottom_pos, bottom_right_pos, bottom_right_pos);
        }
        case 4u: {
            push_quad(top_pos, right_pos, top_right_pos, top_right_pos);
        }
        case 8u: {
            push_quad(top_left_pos, left_pos, top_pos, top_pos);
        }

        // 2 points
        case 3u: {
            push_quad(right_pos, left_pos, bottom_right_pos, bottom_left_pos);
        }
        case 6u: {
            push_quad(top_pos, bottom_pos, top_right_pos, bottom_right_pos);
        }
        case 9u: {
            push_quad(top_left_pos, bottom_left_pos, top_pos, bottom_pos);
        }
        case 12u: {
            push_quad(top_left_pos, left_pos, top_right_pos, right_pos);
        }
        case 5u: {
            push_quad(top_pos, bottom_pos, top_right_pos, right_pos);
            push_quad(top_pos, left_pos, bottom_pos, bottom_left_pos);
        }
        case 10u: {
            push_quad(top_left_pos, bottom_right_pos, top_pos, right_pos);
            push_quad(top_left_pos, left_pos, bottom_right_pos, bottom_pos);
        }

        // 3 points
        case 7u: {
            push_quad(top_pos, bottom_left_pos, top_right_pos, bottom_right_pos);
            push_quad(top_pos, left_pos, bottom_left_pos, bottom_left_pos);
        }
        case 11u: {
            push_quad(top_left_pos, bottom_right_pos, top_pos, right_pos);
            push_quad(top_left_pos, bottom_left_pos, bottom_right_pos, bottom_right_pos);
        }
        case 13u: {
            push_quad(top_left_pos, bottom_pos, top_right_pos, right_pos);
            push_quad(top_left_pos, bottom_left_pos, bottom_pos, bottom_pos);
        }
        case 14u: {
            push_quad(top_left_pos, bottom_pos, top_right_pos, bottom_right_pos);
            push_quad(top_left_pos, left_pos, bottom_pos, bottom_pos);
        }

        // 4 points
        case 15u: {
            push_quad(top_left_pos, bottom_left_pos, top_right_pos, bottom_right_pos);
        }
    }
}

fn push_quad(v0: vec2<f32>, v1: vec2<f32>, v2: vec2<f32>, v3: vec2<f32>) {
    let index = atomicAdd(&quad_indirect_args.instance_count, 1u);
    quads[index * 4u + 0u] = vec2<i32>(v0 * f32(metadata.cell_size)) + metadata.offset;
    quads[index * 4u + 1u] = vec2<i32>(v1 * f32(metadata.cell_size)) + metadata.offset;
    quads[index * 4u + 2u] = vec2<i32>(v2 * f32(metadata.cell_size)) + metadata.offset;
    quads[index * 4u + 3u] = vec2<i32>(v3 * f32(metadata.cell_size)) + metadata.offset;
}

fn lerp(p1: vec2<f32>, p2: vec2<f32>, v1: f32, v2: f32) -> vec2<f32> {
    if v1 == v2 {
        return vec2<f32>(-1.0);
    }

    let t = (1.0 - v1) / (v2 - v1);
    return p1 + t * (p2 - p1);
}