struct VertexInput {
    @location(0) clip_position: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) progress: f32,
    @location(2) show_progress: f32,
    @location(3) level: f32,
};

struct Gradient {
    index: u32,
};

struct Camera {
    position: vec2<f32>,
    scale: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> gradient: Gradient;

@group(0) @binding(1)
var<uniform> camera: Camera;

@group(0) @binding(2)
var gradient_texture: texture_1d<f32>;

@group(0) @binding(3)
var gradient_sample: sampler;

@vertex
fn vertex_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(
        (in.clip_position.xy + camera.position) * camera.scale * 2.0 - 1.0,
        0.0,
        1.0,
    );
    out.level = in.clip_position.z;
    return out;
}

@fragment
fn fragment_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(
	gradient_texture,
	gradient_sample,
        in.level,
    );
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
