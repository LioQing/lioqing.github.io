struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var input_sampler: sampler;

@group(0) @binding(2)
var<uniform> direction: vec4<f32>;

@vertex
fn vert_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let uv = vec2<f32>(
        f32((vertex_index << 1u) & 2u),
        f32(vertex_index & 2u),
    );
    let pos = uv * 2.0 - 1.0;

    return VertexOutput(
        vec4<f32>(pos, 0.0, 1.0),
        uv,
    );
}

@fragment
fn frag_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(input_texture));
    let dir = direction.xy * texel;
    let clamped_uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));

    // 25-tap separable Gaussian weights (sigma ~ 4.0)
    let w0 = 0.09990836;
    let w1 = 0.0968345;
    let w2 = 0.08816882;
    let w3 = 0.07541479;
    let w4 = 0.06059748;
    let w5 = 0.04574138;
    let w6 = 0.0324355;
    let w7 = 0.0216067;
    let w8 = 0.01352113;
    let w9 = 0.00794866;
    let w10 = 0.00438967;
    let w11 = 0.00227733;
    let w12 = 0.00110988;

    var color = textureSample(input_texture, input_sampler, clamped_uv).rgb * w0;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv + dir * 1.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w1;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv - dir * 1.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w1;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv + dir * 2.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w2;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv - dir * 2.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w2;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv + dir * 3.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w3;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv - dir * 3.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w3;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv + dir * 4.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w4;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv - dir * 4.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w4;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv + dir * 5.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w5;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv - dir * 5.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w5;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv + dir * 6.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w6;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv - dir * 6.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w6;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv + dir * 7.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w7;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv - dir * 7.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w7;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv + dir * 8.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w8;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv - dir * 8.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w8;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv + dir * 9.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w9;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv - dir * 9.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w9;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv + dir * 10.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w10;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv - dir * 10.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w10;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv + dir * 11.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w11;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv - dir * 11.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w11;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv + dir * 12.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w12;
    color += textureSample(input_texture, input_sampler, clamp(clamped_uv - dir * 12.0, vec2<f32>(0.0), vec2<f32>(1.0))).rgb * w12;

    return vec4<f32>(color, 1.0);
}
