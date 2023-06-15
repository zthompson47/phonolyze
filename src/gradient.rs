use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InnerGradient {
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

impl Gradient {
    pub fn new(
        label: Option<&str>,
        inner: InnerGradient,
        device: &wgpu::Device,
    ) -> Self {
        Gradient {
            inner,
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(&[inner]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
}
