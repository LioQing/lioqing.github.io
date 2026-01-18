struct FrameMetadata {
    resolution: vec2<u32>,
    top_left: vec2<i32>,
};

struct TextureBlitParams {
    position: vec2<f32>,
    _pad: vec2<f32>,
};

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0)
var src_texture: texture_2d<f32>;

@group(0) @binding(1)
var src_sampler: sampler;

@group(0) @binding(2)
var<uniform> params: TextureBlitParams;

@group(0) @binding(3)
var<uniform> frame: FrameMetadata;

@vertex
fn vert_main(@builtin(vertex_index) vertex_index: u32) -> VertexOut {
    let src_size = vec2<f32>(textureDimensions(src_texture));
    let dst_size = vec2<f32>(frame.resolution);
    let pos = params.position;

    let tl = pos;
    let br = pos + src_size;

    var positions = array<vec2<f32>, 4>(
        vec2<f32>(tl.x, tl.y),
        vec2<f32>(br.x, tl.y),
        vec2<f32>(tl.x, br.y),
        vec2<f32>(br.x, br.y)
    );

    var uvs = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0)
    );

    let pos_px = positions[vertex_index];
    let ndc = vec2<f32>(
        (pos_px.x / dst_size.x) * 2.0 - 1.0,
        1.0 - (pos_px.y / dst_size.y) * 2.0
    );

    var out: VertexOut;
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.uv = uvs[vertex_index];
    return out;
}

@fragment
fn frag_main(in: VertexOut) -> @location(0) vec4<f32> {
    return textureSample(src_texture, src_sampler, in.uv);
}
