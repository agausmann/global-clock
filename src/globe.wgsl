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
    out.position = vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

[[block]]
struct Uniforms {
    angle: f32;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;
[[group(0), binding(1)]]
var globe_sampler: sampler;
[[group(0), binding(2)]]
var globe_texture: texture_2d<f32>;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    // Map 0.0..1.0 to -1.0..1.0
    var x: f32 = in.uv.x * 2.0 - 1.0;
    // Map 0.0..1.0 to 1.0..-1.0
    var y: f32 = 1.0 - in.uv.y * 2.0;

    var radius: f32 = sqrt(x * x + y * y);
    var angle: f32 = atan2(y, x) + uniforms.angle;

    // Note this is in 0.0..1.0, not degrees
    var lat_lon: vec2<f32> = vec2<f32>(-angle / 6.283185, 1.0 - radius);
    var globe_color: vec4<f32> = textureSample(globe_texture, globe_sampler, lat_lon);

    if (radius <= 1.0) {
        return globe_color;
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}
