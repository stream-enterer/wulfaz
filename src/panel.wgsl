struct Uniforms {
    projection: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) size_px: vec2<f32>,
    @location(3) bg_color: vec4<f32>,
    @location(4) border_color: vec4<f32>,
    @location(5) border_width: f32,
    @location(6) shadow_width: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) size_px: vec2<f32>,
    @location(2) bg_color: vec4<f32>,
    @location(3) border_color: vec4<f32>,
    @location(4) border_width: f32,
    @location(5) shadow_width: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.size_px = in.size_px;
    out.bg_color = in.bg_color;
    out.border_color = in.border_color;
    out.border_width = in.border_width;
    out.shadow_width = in.shadow_width;
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Distance to nearest edge in pixels
    let dx = min(in.uv.x * in.size_px.x, (1.0 - in.uv.x) * in.size_px.x);
    let dy = min(in.uv.y * in.size_px.y, (1.0 - in.uv.y) * in.size_px.y);
    let dist = min(dx, dy);

    let bg_linear = srgb_to_linear3(in.bg_color.rgb);
    let border_linear = srgb_to_linear3(in.border_color.rgb);

    var color: vec3<f32>;
    var alpha: f32;

    if dist < in.border_width {
        // Border stroke
        color = border_linear;
        alpha = in.border_color.a;
    } else if dist < in.border_width + in.shadow_width {
        // Inner shadow: lerp from darkened bg to bg
        let t = (dist - in.border_width) / in.shadow_width;
        let darkened = bg_linear * 0.6;
        color = mix(darkened, bg_linear, t);
        alpha = in.bg_color.a;
    } else {
        // Background fill
        color = bg_linear;
        alpha = in.bg_color.a;
    }

    // Premultiplied alpha output
    return vec4<f32>(color * alpha, alpha);
}
