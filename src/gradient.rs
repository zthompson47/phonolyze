use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InnerGradient {
    pub grad: [[f32; 4]; 256],
    pub r: [f32; 4],
    pub g: [f32; 4],
    pub b: [f32; 4],
    pub a: [f32; 4],
    pub domain: [f32; 4],
}

#[derive(Debug)]
pub struct Gradient {
    #[allow(dead_code)]
    inner: InnerGradient,
    buffer: wgpu::Buffer,
}

fn grad() -> [[f32; 4]; 256] {
    let mut grad = [[0.0, 0.0, 0.0, 0.0]; 256];
    colorgrad::rainbow()
        .colors(256)
        .iter()
        .map(colorgrad::Color::to_linear_rgba)
        .collect::<Vec<_>>()
        .iter()
        .enumerate()
        .for_each(|(i, c)| {
            grad[i] = [c.0 as f32, c.1 as f32, c.2 as f32, c.3 as f32];
        });
    grad
}

impl Gradient {
    pub fn new(label: Option<&str>, mut inner: InnerGradient, device: &wgpu::Device) -> Self {
        inner.grad = grad();
        Gradient {
            inner,
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(&[inner]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }

    pub fn update(&mut self, mut inner: InnerGradient, queue: &wgpu::Queue) {
        inner.grad = grad();
        self.inner = inner;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
}
