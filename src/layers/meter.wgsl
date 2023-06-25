struct VertexInput {
    @location(0) clip_position: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) coords: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> progress: vec4<f32>;

@vertex
fn vertex_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.clip_position.xy, 0.0, 1.0);
    out.coords = vec2<f32>(in.clip_position.xy);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let clipnorm = (in.coords.x + 1.0) / 2.0;
    let prognorm = progress.x / progress.y;
    var diff = abs(prognorm - clipnorm);
    var p = clamp(0.2, 1.0, 1.0 - smoothstep(0.0, 0.0025, diff));
    return vec4<f32>(0.0, 0.0, 0.0, p);
}
