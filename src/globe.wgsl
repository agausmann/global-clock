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
    min_latitude: f32;
    max_latitude: f32;
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

fn lerp(factor: f32, a: f32, b: f32) -> f32 {
    return a * (1.0 - factor) + b * factor;
}

fn lerp4(factor: f32, a: vec4<f32>, b: vec4<f32>) -> vec4<f32> {
    return a * (1.0 - factor) + b * factor;
}

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    // Map 0.0..1.0 to -1.0..1.0
    var x: f32 = in.uv.x * 2.0 - 1.0;
    // Map 0.0..1.0 to 1.0..-1.0
    var y: f32 = 1.0 - in.uv.y * 2.0;

    var radius: f32 = sqrt(x * x + y * y);
    // Positive remainder
    var abs_angle: f32 = -atan2(y, x);

    // Note this is in radians, not degrees
    var lon_lat: vec2<f32> = vec2<f32>(abs_angle, lerp(radius, uniforms.min_latitude, uniforms.max_latitude));

    var globe_ray: vec3<f32> = vec3<f32>(
        cos(lon_lat.y) * cos(lon_lat.x),
        cos(lon_lat.y) * sin(lon_lat.x),
        sin(lon_lat.y),
    );

    var sun_ray: vec3<f32> = vec3<f32>(0.0, -1.0, 0.0);

    var night_day_blend: f32;
    // abs_angle is in range -TAU/2..TAU/2
    if (abs_angle > 0.0) {
        night_day_blend = 0.0;
    } else {
        night_day_blend = 1.0;
    }
    // override
    night_day_blend = 1.0 / (1.0 + exp(-20.0 * dot(sun_ray, globe_ray)));

    var rotated_angle: f32 = abs_angle + uniforms.rotation;
    // Note this is in 0.0..1.0, not degrees
    var tex_coord: vec2<f32> = vec2<f32>((lon_lat.x - uniforms.rotation) / TAU, 0.5 - lon_lat.y / TAU * 2.0);
    var day_color: vec4<f32> = textureSample(globe_day_texture, globe_sampler, tex_coord);
    var night_color: vec4<f32> = textureSample(globe_night_texture, globe_sampler, tex_coord);
    var globe_color: vec4<f32> = lerp4(night_day_blend, night_color, day_color);

    if (radius <= 1.0) {
        return globe_color;
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}
