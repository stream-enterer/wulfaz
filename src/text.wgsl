struct Uniforms {
    projection: mat4x4<f32>,
    fg_color: vec4<f32>,
    bg_color: vec4<f32>,
    gamma_adj: f32,
    contrast: f32,
    _pad: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var atlas_tex: texture_2d<f32>;
@group(0) @binding(2) var atlas_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

// IEC 61966-2-1 sRGB to linear conversion
fn srgb_to_linear(s: f32) -> f32 {
    if s <= 0.04045 {
        return s / 12.92;
    }
    return pow((s + 0.055) / 1.055, 2.4);
}

fn srgb_to_linear3(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        srgb_to_linear(c.r),
        srgb_to_linear(c.g),
        srgb_to_linear(c.b),
    );
}

// BT.709 luminance
fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample glyph alpha from R8Unorm atlas
    let a = textureSample(atlas_tex, atlas_sampler, in.uv).r;

    // Convert sRGB uniforms to linear
    let fg_linear = srgb_to_linear3(uniforms.fg_color.rgb);
    let bg_linear = srgb_to_linear3(uniforms.bg_color.rgb);

    // BT.709 luminance for contrast adjustment
    let fg_lum = luminance(fg_linear);
    let bg_lum = luminance(bg_linear);

    // Kitty contrast adjustment (§7.3)
    let adjustment = (1.0 - fg_lum + bg_lum) * 0.5;
    let adjusted_alpha = clamp(
        mix(a, pow(a, uniforms.gamma_adj), adjustment) * uniforms.contrast,
        0.0, 1.0
    );

    // Premultiplied alpha output — sRGB surface auto-converts linear→sRGB
    return vec4<f32>(fg_linear * adjusted_alpha, adjusted_alpha);
}
