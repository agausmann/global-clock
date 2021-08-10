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
    rotation: f32;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;
[[group(0), binding(1)]]
var globe_sampler: sampler;
[[group(0), binding(2)]]
var globe_day_texture: texture_2d<f32>;
[[group(0), binding(3)]]
var globe_night_texture: texture_2d<f32>;

var TAU: f32 = 6.283185;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    // Map 0.0..1.0 to -1.0..1.0
    var x: f32 = in.uv.x * 2.0 - 1.0;
    // Map 0.0..1.0 to 1.0..-1.0
    var y: f32 = 1.0 - in.uv.y * 2.0;

    var radius: f32 = sqrt(x * x + y * y);
    // Positive remainder
    var abs_angle: f32 = atan2(y, x);
    var rotated_angle: f32 = abs_angle + uniforms.rotation;

    var night_day_blend: f32;

    // abs_angle is in range -TAU/2..TAU/2
    if (abs_angle < 0.0) {
        night_day_blend = 0.0;
    } else {
        night_day_blend = 1.0;
    }
        
    // Note this is in 0.0..1.0, not degrees
    var lat_lon: vec2<f32> = vec2<f32>(-rotated_angle / TAU, 1.0 - radius);
    var day_color: vec4<f32> = textureSample(globe_day_texture, globe_sampler, lat_lon);
    var night_color: vec4<f32> = textureSample(globe_night_texture, globe_sampler, lat_lon);
    var globe_color: vec4<f32> = day_color * night_day_blend + night_color * (1.0 - night_day_blend);

    if (radius <= 1.0) {
        return globe_color;
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}
