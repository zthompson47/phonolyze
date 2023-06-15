struct VertexInput {
    @location(0) clip_position: vec4<f32>,
    //@location(1) level: f32,
    //@location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) level: f32,
    //@location(1) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.clip_position.xy, 0.0, 1.0);
    out.level = in.clip_position.z;
    //out.color = in.color;
    return out;
    //let lvl = in.level;
    //out.color = vec4<f32>(1.0, 0.0, 0.0, 0.5);
}

struct Gradient {
    r: vec4<f32>,
    g: vec4<f32>,
    b: vec4<f32>,
    a: vec4<f32>,
    domain: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> gradient: Gradient;

//@group(0) @binding(1)
//var s_diffuse: sampler;
//@group(0) @binding(2)
//var<uniform> scale: vec4<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //let grad = mat4x4<f32>(
    //    vec4<f32>(1.0, 0.0, 0.0, 1.0),
    //    vec4<f32>(1.0, 0.0, 1.0, 0.0),
    //    vec4<f32>(1.0, 1.0, 0.0, 0.0),
    //    vec4<f32>(1.0, 0.8, 1.0, 1.0),
    //);
    //let domain = vec4<f32>(-150.0, -80.0, -40.0, 0.0);

    let grad = mat4x4<f32>(
        gradient.r,
        gradient.g,
        gradient.b,
        gradient.a
    );
    let domain = gradient.domain;

    //return grad_at(transpose(grad), domain, in.level);
    return grad_at(grad, domain, in.level);

    //return in.color;
    //return vec4<f32>(in.level, in.level, in.level, 1.0);
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
            //    if at < domain_max && at >= domain_min {
            //        let extent_factor = (at - domain_min) / domain_norm;
            //        let channel_extent = channel_diff * extent_factor;
            //        result_color[i] = channel_min + channel_extent;
            //    }
            //}
        }
    }

    return result_color;
}

            //let domain_min = domain[j] / domain_norm;
            //let channel_min = channel[j];
            //let domain_max = domain[j + 1] / domain_norm;
            //let channel_max = channel[j + 1];
            //let domain_diff = domain_max - domain_min;
            //let channel_diff = channel_max - channel_min;
            //
            //if at < domain_max && at >= domain_min {
            //    let extent_factor = at - domain_min;
            //    let channel_extent = channel_diff * extent_factor;
            //    result_color[i] = channel_extent;
            //}
