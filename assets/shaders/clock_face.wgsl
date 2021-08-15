[[group(0), binding(1)]]
var t_sampler: sampler;
[[group(0), binding(2)]]
var texture: texture_2d<f32>;

[[block]]
struct Viewport {
    proj: mat4x4<f32>;
};

[[group(1), binding(0)]]
var<uniform> viewport: Viewport;

struct VertexInput {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] uv: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = viewport.proj * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    // Temporary crash fix, see gfx-rs/wgpu#1803
    var _: mat4x4<f32> = viewport.proj;

    return textureSample(texture, t_sampler, in.uv);
}
