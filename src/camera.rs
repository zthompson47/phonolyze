use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InnerCamera {
    pub position: [f32; 2],
    pub scale: [f32; 2],
    pub progress: [f32; 2],
}

#[derive(Debug)]
pub struct Camera {
    #[allow(dead_code)]
    pub inner: InnerCamera,
    buffer: wgpu::Buffer,
}

impl Camera {
    pub fn new(label: Option<&str>, inner: InnerCamera, device: &wgpu::Device) -> Self {
        Self {
            inner,
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(&[inner]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }

    pub fn update(&mut self, inner: InnerCamera, queue: &wgpu::Queue) {
        self.inner = inner;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn move_horizontal(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.position[0] += amount;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn scale_horizontal(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.scale[0] += amount;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn move_vertical(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.position[1] += amount;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn scale_vertical(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.scale[1] += amount;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn update_progress(&mut self, progress: f32, queue: &wgpu::Queue) {
        self.inner.progress[0] = progress;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
}
