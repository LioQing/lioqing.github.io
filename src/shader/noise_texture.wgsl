// Vertex

@vertex
fn vert_main(@builtin(vertex_index) vi: u32) -> FragmentInput {
    var out: FragmentInput;

    out.uv = vec2<f32>(
        f32((vi << 1u) & 2u),
        f32(vi & 2u),
    );
    out.clip_pos = vec4<f32>(out.uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv.y = 1.0 - out.uv.y;

    return out;
}

// Fragment

struct FragmentInput {
    @location(0) uv: vec2<f32>,

    @builtin(position) clip_pos: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> seed: u32;

@fragment
fn frag_main(in: FragmentInput) -> @location(0) vec4<f32> {
    let pos = in.clip_pos.xy;
    var x = u32(pos.x);
    var y = u32(pos.y);
    var h = seed + x * 374761393 + y * 668265263;
    h = (h ^ (h >> 13)) * 1274126177;
    h = (h ^ (h >> 16));
    let rand = f32(h) / 4294967295.0;
    return vec4<f32>(rand, rand, rand, 1.0);
}