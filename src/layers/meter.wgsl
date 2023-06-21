struct VertexInput {
    @location(0) clip_position: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> progress: vec4<f32>;

@vertex
fn vertex_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let p = progress.x;
    out.clip_position = vec4<f32>(in.clip_position.xy, 0.0, 1.0);
    out.color = vec4<f32>(0.0, 0.0, 0.5, p);
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
