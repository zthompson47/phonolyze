struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

/*
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var<uniform> scale: vec4<f32>;
*/

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //var tex_coord = vec2<f32>(0.0, 0.0);
    //tex_coord.x = in.tex_coords.x * scale.x;
    //tex_coord.y = in.tex_coords.y * scale.y;
    //tex_coord.x = in.tex_coords.x * scale.z;
    //tex_coord.y = in.tex_coords.y * scale.w;
    //return textureSample(t_diffuse, s_diffuse, tex_coord);
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
