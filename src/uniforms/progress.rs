use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InnerProgress {
    pub position: [f32; 4],
}

#[derive(Debug)]
pub struct Progress {
    pub inner: InnerProgress,
    buffer: wgpu::Buffer,
}

impl Progress {
    pub fn new(device: &wgpu::Device) -> Self {
        let inner = InnerProgress::default();
        Self {
            inner,
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Progress"),
                contents: bytemuck::cast_slice(&[inner]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }

    pub fn bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ProgressPass"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    pub fn update_position(&mut self, position: f32, length: f32, queue: &wgpu::Queue) {
        self.inner.position[0] = position;
        self.inner.position[1] = length;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
}
