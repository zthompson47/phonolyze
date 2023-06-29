use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InnerCamera {
    position: [f32; 2],
    scale: [f32; 2],
}

#[derive(Debug)]
pub struct Camera {
    inner: InnerCamera,
    buffer: wgpu::Buffer,
}

impl Camera {
    pub fn new(device: &wgpu::Device) -> Self {
        let inner = InnerCamera {
            position: [0.0, 0.0],
            scale: [1.0, 1.0],
        };

        Self {
            inner,
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("CameraUniform"),
                contents: bytemuck::cast_slice(&[inner]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }

    pub fn zero(&mut self, queue: &wgpu::Queue) {
        self.inner.position[0] = 0.0;
        self.inner.position[1] = 0.0;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn fill(&mut self, queue: &wgpu::Queue) {
        self.inner.position[0] = 0.0;
        self.inner.position[1] = 0.0;
        self.inner.scale[0] = 1.0;
        self.inner.scale[1] = 1.0;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn move_x(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.position[0] += amount;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn move_y(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.position[1] += amount;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn scale_x(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.scale[0] += self.inner.scale[0] * amount;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn scale_y(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.scale[1] += self.inner.scale[1] * amount;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn bind_group_entry(index: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: index,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
}
