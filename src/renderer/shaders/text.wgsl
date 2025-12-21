// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct Uniforms {
    transform: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.color = model.color;
    out.clip_position = uniforms.transform * vec4<f32>(model.position, 0.0, 1.0);
    return out;
}

// Fragment shader

@group(1) @binding(0)
var t_texture: texture_2d<f32>;
@group(1) @binding(1)
var t_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Check if this is a solid color quad (tex_coords at origin means no texture)
    // For background colors and borders, we use solid color without texture sampling
    if (in.tex_coords.x == 0.0 && in.tex_coords.y == 0.0) {
        return in.color;
    }

    // For text glyphs, sample the alpha from the glyph atlas
    let alpha = textureSample(t_texture, t_sampler, in.tex_coords).r;
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
