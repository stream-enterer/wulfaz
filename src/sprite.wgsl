// Sprite rendering shader (UI-202b).
// Textured quads sampling an RGBA atlas with per-vertex tint.
// Same projection convention as text.wgsl and panel.wgsl.

struct Uniforms {
    projection: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var atlas_tex: texture_2d<f32>;
@group(0) @binding(2) var atlas_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tint: vec4<f32>,      // sRGB tint color (multiplied with texture)
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(input.position, 0.0, 1.0);
    out.uv = input.uv;
    out.tint = input.tint;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(atlas_tex, atlas_sampler, input.uv);

    // Convert sRGB tint to linear.
    let tint_linear = vec4<f32>(
        pow(input.tint.r, 2.2),
        pow(input.tint.g, 2.2),
        pow(input.tint.b, 2.2),
        input.tint.a,
    );

    // Multiply texture color (already linear from Rgba8UnormSrgb) by tint.
    let color = tex * tint_linear;

    // Output premultiplied alpha (surface is sRGB, auto-converts back).
    return vec4<f32>(color.rgb * color.a, color.a);
}
