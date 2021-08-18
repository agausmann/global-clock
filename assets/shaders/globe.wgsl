[[block]]
struct Uniforms {
    local_transform: mat4x4<f32>;
    rotation: f32;
    axial_tilt: f32;
    min_latitude: f32;
    max_latitude: f32;
    deflection_point: vec2<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;
[[group(0), binding(1)]]
var globe_sampler: sampler;
[[group(0), binding(2)]]
var globe_day_texture: texture_2d<f32>;
[[group(0), binding(3)]]
var globe_night_texture: texture_2d<f32>;

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
    out.position = viewport.proj * uniforms.local_transform * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

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
    var abs_angle: f32 = -atan2(y, x);

    // Note these are in radians, not degrees
    var longitude: f32 = abs_angle;
    var latitude: f32;
    if (radius < uniforms.deflection_point.x) {
        latitude = lerp(
            radius / uniforms.deflection_point.x,
            uniforms.min_latitude,
            uniforms.deflection_point.y,
        );
    } else {
        latitude = lerp(
            (radius - uniforms.deflection_point.x) / (1.0 - uniforms.deflection_point.x),
            uniforms.deflection_point.y,
            uniforms.max_latitude,
        );
    }

    // 3D space for light calculations:
    // - Equator lies in the XY plane
    // - Positive Z is toward the north pole
    // - Positive Y is toward the sun
    var globe_ray: vec3<f32> = vec3<f32>(
        cos(latitude) * cos(longitude),
        cos(latitude) * sin(longitude),
        sin(latitude),
    );
    var sun_ray: vec3<f32> = vec3<f32>(0.0, cos(uniforms.axial_tilt), sin(uniforms.axial_tilt));

    var night_day_blend: f32 = 1.0 / (1.0 + exp(-20.0 * dot(sun_ray, globe_ray)));

    var tex_coord: vec2<f32> = vec2<f32>(
        (longitude - uniforms.rotation) / TAU,
        0.5 - latitude / TAU * 2.0,
    );
    var day_color: vec4<f32> = textureSample(globe_day_texture, globe_sampler, tex_coord);
    var night_color: vec4<f32> = textureSample(globe_night_texture, globe_sampler, tex_coord);

    var ambient: vec4<f32> = lerp4(night_day_blend, night_color, 0.7 * day_color);
    var diffuse: vec4<f32> = 0.3 * day_color * max(0.0, dot(sun_ray, globe_ray));

    if (radius <= 1.0) {
        return ambient + diffuse;
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}
