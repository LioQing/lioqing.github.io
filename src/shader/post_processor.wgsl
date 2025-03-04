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
var wave_texture: texture_2d<f32>;

@group(0) @binding(1)
var wave_sampler: sampler;

@group(0) @binding(2)
var<uniform> base_color: u32;

@group(0) @binding(3)
var noise_texture: texture_2d<f32>;

@fragment
fn frag_main(in: FragmentInput) -> @location(0) vec4<f32> {
    let noise_tex_dims = textureDimensions(noise_texture);
    let noise = textureLoad(noise_texture, vec2<u32>(in.clip_pos.xy) % noise_tex_dims, 0).r;

    let wave_tex_sim = textureDimensions(wave_texture);
    let step = vec2<f32>(0.02) / vec2<f32>(wave_tex_sim);
    let amp = textureSample(wave_texture, wave_sampler, in.uv).r;
    let amp_right = textureSample(wave_texture, wave_sampler, in.uv + vec2<f32>(step.x, 0.0)).r;
    let amp_up = textureSample(wave_texture, wave_sampler, in.uv + vec2<f32>(0.0, step.y)).r;
    
    let base = unpack4x8unorm(base_color).rgb;
    
    let normal = normalize(vec3<f32>(
        -(amp_right - amp) / step.x,
        -(amp_up - amp) / step.y,
        1.0
    ));
    
    // light_dir = vec3(2.0, 2.0, 1.0).normalize();
    const light_dir = vec3<f32>(0.6666667, 0.6666667, 0.33333334);
    const perp_light_intensity = 0.5;
    const view_dir = vec3<f32>(0.0, 0.0, 1.0);
    let diffuse = max(dot(normal, light_dir), 0.0);
    let reflect_dir = reflect(-light_dir, normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    
    const ambient = 0.3;
    let light_intensity = max(ambient + diffuse * 0.6 + spec * 0.3 - perp_light_intensity, 0.0);

    let slope_factor = length(vec2<f32>(normal.x, normal.y));
    let noise_factor = noise * slope_factor;

    let light_scale = select(1.0, 10.0, base.r < 0.5);
    let light_scaled = (1.0 + (light_intensity * light_scale + amp * 0.3) * noise_factor);
    let light = select(
        1.0 / light_scaled,
        light_scaled,
        base.r < 0.5
    );

    let color = base * light;
    
    return vec4<f32>(color, 1.0);
}