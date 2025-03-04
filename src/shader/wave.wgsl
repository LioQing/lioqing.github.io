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

struct Action {
    pos: vec2<f32>,
    force: f32,
}
@group(0) @binding(0)
var<storage, read_write> actions: array<Action, 2>; // actions[0] = current, actions[1] = previous

@group(0) @binding(1)
var wave_t0: texture_2d<f32>; // wave(t-2)

@group(0) @binding(2)
var wave_t1: texture_2d<f32>; // wave(t-1)

@group(0) @binding(3)
var logo_texture: texture_2d<f32>;

@group(0) @binding(4)
var logo_sampler: sampler;

@fragment
fn frag_main(in: FragmentInput) -> @location(0) vec4<f32> {
    let tex_dims = textureDimensions(wave_t0);
    let tex_coords = vec2<u32>(in.clip_pos.xy);

    // Logo
    const max_amp = 4.0;
    const padding = 16; // 32 * 2 * resolution_scale = 32 * 2 * 0.25
    let right_half_start = tex_dims.x / 2;
    let logo_size = min(tex_dims.x / 2 - 2 * padding, tex_dims.y - 2 * padding);
    let logo_pos = vec2<u32>(
        right_half_start + (tex_dims.x / 2 - logo_size) / 2,
        (tex_dims.y - logo_size) / 2,
    );

    let local_coords = vec2<u32>(tex_coords - logo_pos);
    let logo_uv = vec2<f32>(local_coords) / f32(logo_size);
    let logo_color = textureSample(logo_texture, logo_sampler, logo_uv);

    if all(tex_coords >= logo_pos) &&
        all(tex_coords < logo_pos + vec2<u32>(logo_size)) &&
        logo_color.a > 0.5 {
        return vec4<f32>(max_amp / 2.0, 0.0, 0.0, 0.0);
    }

    if any(tex_coords == vec2<u32>(0u)) || any(tex_coords == tex_dims - vec2<u32>(1u)) {
        return vec4<f32>(0.0);
    }

    // Action
    const action_radius_factor = 0.05;
    let radius = f32(tex_dims.y) * action_radius_factor;

    let diff_0 = floor(actions[0].pos) + vec2<f32>(0.5) - in.clip_pos.xy;
    let dist_sq_0 = dot(diff_0, diff_0);
    if actions[0].force != 0.0 && dist_sq_0 < radius * radius {
        let std_dev = radius / 4.0;
        let g = 0.398942 * exp(-0.5 * dist_sq_0 / (std_dev * std_dev));

        const action_force_factor = 2.0;

        actions[1] = actions[0];
        return vec4<f32>(actions[0].force * g * action_force_factor, 0.0, 0.0, 0.0);
    }

    const tension = 0.6; // <= sqrt(2) / 2 = 0.707
    let amp_x0_y0_t0 = textureLoad(wave_t0, tex_coords, 0).r;
    let amp_x0_y0_t1 = textureLoad(wave_t1, tex_coords, 0).r;
    let amp_xp1_y0_t1 = textureLoad(wave_t1, tex_coords + vec2<u32>(1u, 0u), 0).r;
    let amp_x0_yp1_t1 = textureLoad(wave_t1, tex_coords + vec2<u32>(0u, 1u), 0).r;
    let amp_xn1_y0_t1 = textureLoad(wave_t1, tex_coords - vec2<u32>(1u, 0u), 0).r;
    let amp_x0_yn1_t1 = textureLoad(wave_t1, tex_coords - vec2<u32>(0u, 1u), 0).r;

    // Previouos action
    let diff_1 = floor(actions[1].pos) + vec2<f32>(0.5) - in.clip_pos.xy;
    let dist_sq_1 = dot(diff_1, diff_1);
    if actions[1].force != 0.0 && dist_sq_1 < radius * radius {
        const init_vel_factor = 0.0;
        let amp = amp_x0_y0_t1
            + init_vel_factor * tension * tension * (
                amp_xp1_y0_t1 + amp_x0_yp1_t1 + amp_xn1_y0_t1 + amp_x0_yn1_t1
                - 4.0 * amp_x0_y0_t1
            );
        
        actions[1] = actions[0];
        return vec4<f32>(amp, 0.0, 0.0, 0.0);
    }

    // Wave equation
    const damp = 0.995; // <= 1
    let amp = clamp(
        amp_x0_y0_t1
        + (amp_x0_y0_t1 - amp_x0_y0_t0 * damp)
        + tension * tension * (
            amp_xp1_y0_t1 + amp_x0_yp1_t1 + amp_xn1_y0_t1 + amp_x0_yn1_t1
            - 4.0 * amp_x0_y0_t1
        ),
        -max_amp,
        max_amp
    );

    actions[1] = actions[0];
    return vec4<f32>(amp * damp, 0.0, 0.0, 0.0);
}