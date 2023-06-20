struct Vertex {
    clip_position: vec2<f32>,
    level: f32,
    pad: f32,
};

struct VertexInput {
    @location(0) clip_position: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) progress: f32,
    @location(2) show_progress: f32,
};

struct Gradient {
    rgba: mat4x4<f32>,
    domain: vec4<f32>,
};

struct Camera {
    position: vec2<f32>,
    scale: vec2<f32>,
    progress: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> gradient: Gradient;
@group(0) @binding(1)
var<uniform> camera: Camera;

@vertex
fn vertex_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Clip position
    let pos = (in.clip_position.xy + camera.position) * camera.scale;
    out.clip_position = vec4<f32>(pos, 0.0, 1.0);

    // Gradient color
    out.color = grad_at(gradient.rgba, gradient.domain, in.clip_position.z);

    // Progress bar
    //let normed_progress = camera.progress.x / camera.progress.y;
    //let normed_clip = ((in.clip_position.x + 1.0) / 2.0); // * camera.progress.y;
    //out.progress = f32(abs(normed_progress - normed_clip));
    //out.show_progress = camera.progress.z;

    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //var color = in.color;

    // Progress bar
    //if in.show_progress > 0.5 {
    //    let p = smoothstep(0.0, 0.01, in.progress);
    //    color = vec4f(color.rgb * p, color.a);
    //}

    return in.color;
}

fn grad_at(grad: mat4x4<f32>, domain: vec4<f32>, at: f32) -> vec4<f32> {
    var grad = grad;
    var result_color = vec4<f32>();

    let domain_norm = domain[3] - domain[0];

    for (var i: i32 = 0; i < 4; i++) {
        let channel = grad[i];

        for (var j: i32 = 0; j < 3; j++) {
            let domain_min = domain[j];
            let domain_max = domain[j + 1];
            let domain_diff = domain_max - domain_min;

            let channel_min = channel[j];
            let channel_max = channel[j + 1];
            let channel_diff = channel_max - channel_min;

            //if j == 2 {
            if at >= domain_min && at <= domain_max {
                let extent_factor = (at - domain_min) / domain_diff;
                let channel_extent = channel_diff * extent_factor;
                result_color[i] = channel_min + channel_extent;
            }
            //} else {
        }
    }

    return result_color;
}
