struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_position: vec2<f32>,
    @location(1) in_vertex_index: u32,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    let r = 1.0;
    let z = 0.0;
    var vertices = array<vec3<f32>, 6>(
        vec3<f32>(-1.0, 1.0, z),
        vec3<f32>(-1.0, -1.0, z),
        vec3<f32>(r, 1.0, z),

        vec3<f32>(r, 1.0, z),
        vec3<f32>(-1.0, -1.0, z),
        vec3<f32>(r, -1.0, z),
    );
    var texture_positions = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 1.0),

        //vec2<f32>(1.0, 1.0),
        //vec2<f32>(0.0, 0.0),
        //vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(0.0, 1.0),
    );
    let vert = vertices[in_vertex_index];
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vert, 1.0);
    out.texture_position = texture_positions[in_vertex_index];
    out.in_vertex_index = in_vertex_index;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var<uniform> scale: vec4<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_pos = vec2<f32>(0.0, 0.0);

    // Flip image over and align top-left of image with top-left of window,
    // then scale it to actual size.
    if in.in_vertex_index < u32(3) {
        tex_pos.x = in.texture_position.x * scale.x;
        tex_pos.y = ((1.0 - in.texture_position.y) * scale.y);
    } else {
        tex_pos.x = in.texture_position.y * scale.x;
        tex_pos.y = ((1.0 - in.texture_position.x) * scale.y);
    }

    return vec4<f32>(textureSample(t_diffuse, s_diffuse, tex_pos).xyz, 1.0);
}
