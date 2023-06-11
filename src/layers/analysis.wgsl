struct VertexInput {
    @location(0) clip_position: vec4<f32>,
    //@location(1) level: f32,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) level: f32,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    //out.clip_position = vec4<f32>(in.clip_position, 0.0, 1.0);
    out.clip_position = vec4<f32>(
        //in.clip_position.x * 2.0 - 1.0,
        //in.clip_position.y * 2.0 - 1.0,
        //in.clip_position.x * 4.0 - 2.0,
        //in.clip_position.y * 4.0 - 2.0,
        in.clip_position.x,
        in.clip_position.y,
        0.0,
        1.0
    );
    //out.level = in.level;
    out.level = 0.0;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //var tex_coord = vec2<f32>(0.0, 0.0);
    //tex_coord.x = in.tex_coords.x * scale.x;
    //tex_coord.y = in.tex_coords.y * scale.y;
    //tex_coord.x = in.tex_coords.x * scale.z;
    //tex_coord.y = in.tex_coords.y * scale.w;
    //return textureSample(t_diffuse, s_diffuse, tex_coord);
    //return vec4<f32>(in.level, 0.0, 0.0, in.level);

    return in.color;
    //return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}
