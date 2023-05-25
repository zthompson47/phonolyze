use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InnerScale {
    x_norm: f32,
    y_norm: f32,
    x_val: f32,
    y_val: f32,
}

#[derive(Debug)]
pub struct Scale {
    window_size: PhysicalSize<u32>,
    image_size: PhysicalSize<u32>,
    inner: InnerScale,
    buffer: wgpu::Buffer,
    max_x_val: f32,
    max_y_val: f32,
}

impl Scale {
    pub fn new(
        label: Option<&str>,
        max_x_val: f32,
        max_y_val: f32,
        window_size: PhysicalSize<u32>,
        image_size: PhysicalSize<u32>,
        device: &wgpu::Device,
    ) -> Self {
        let inner = InnerScale {
            x_norm: 1.,
            y_norm: 1.,
            x_val: inv_quint_ease_in(1., 0., max_x_val),
            y_val: inv_quint_ease_in(1., 0., max_y_val),
        };

        Scale {
            inner,
            window_size,
            image_size,
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(&[inner]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            max_x_val,
            max_y_val,
        }
    }

    pub fn binding_resource(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }

    //fn unscale(&mut self) {}
    pub fn center(&mut self, queue: &wgpu::Queue) {
        self.inner.x_val = (self.window_size.width as f32 / self.image_size.width as f32)
            .clamp(0., self.max_x_val);
        self.inner.y_val = (self.window_size.height as f32 / self.image_size.height as f32)
            .clamp(0., self.max_y_val);
        self.inner.x_norm = inv_quint_ease_in(self.inner.x_val, 0., self.max_x_val);
        self.inner.y_norm = inv_quint_ease_in(self.inner.y_val, 0., self.max_y_val);

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn scale_x(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.x_norm = (self.inner.x_norm + amount).clamp(0., 1.);
        self.inner.x_val = quint_ease_in(self.inner.x_norm, 0., self.max_x_val);

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn scale_y(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.y_norm = (self.inner.y_norm + amount).clamp(0., 1.);
        self.inner.y_val = quint_ease_in(self.inner.y_norm, 0., self.max_y_val);

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>, queue: &wgpu::Queue) {
        let ratio_x = new_size.width as f32 / self.window_size.width as f32;
        let ratio_y = new_size.height as f32 / self.window_size.height as f32;

        self.window_size = new_size;
        self.inner.x_val = (self.inner.x_val * ratio_x).clamp(0., self.max_x_val);
        self.inner.y_val = (self.inner.y_val * ratio_y).clamp(0., self.max_y_val);
        /*
        self.inner.x_val = ((self.window_size.width as f32 / self.image_size.width as f32)
            * self.inner.x_norm)
            .clamp(0., self.max_y_val);
        self.inner.y_val = ((self.window_size.height as f32 / self.image_size.height as f32)
            * self.inner.y_norm)
            .clamp(0., self.max_y_val);
            */
        self.inner.x_norm = inv_quint_ease_in(self.inner.x_val, 0., self.max_x_val);
        self.inner.y_norm = inv_quint_ease_in(self.inner.y_val, 0., self.max_y_val);

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }
}

fn quint_ease_in(x: f32, min: f32, max: f32) -> f32 {
    assert!(x >= 0.);
    assert!(x <= 1.0);
    (max - min) * x.powi(5) + min
}

fn inv_quint_ease_in(x: f32, min: f32, max: f32) -> f32 {
    assert!(x >= min);
    assert!(x <= max);
    ((x - min) / (max - min)).powf(1. / 5.)
}
