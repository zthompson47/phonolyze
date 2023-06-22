#![warn(missing_docs)]
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::ease;

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
    /// Size of viewing window
    window_size: PhysicalSize<u32>,
    /// Size of displayed image
    image_size: PhysicalSize<u32>,
    /// Scaling factor uniforms for wgpu
    inner: InnerScale,
    /// Buffer for wgpu updates
    buffer: wgpu::Buffer,
    /// Maximum horizontal scale-out factor
    max_x_val: f32,
    /// Maximum vertical scale-out factor
    max_y_val: f32,
}

impl Scale {
    /// Create a new scaling system.
    ///
    /// # Arguments
    ///
    /// * `label` - bla
    /// * `max_x_val` - bla
    /// * `max_y_val` - bla
    /// * `window_size` - bla
    /// * `image_size` - bla
    /// * `device` - bla
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
            x_val: ease::inv_quint_ease_in(1., 0., max_x_val),
            y_val: ease::inv_quint_ease_in(1., 0., max_y_val),
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

    /// Return buffer as bind group resource
    pub fn binding_resource(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }

    /// Reset scale to one image pixel per window pixel.
    pub fn unscale(&mut self, queue: &wgpu::Queue) {
        self.inner.x_val = (self.window_size.width as f32 / self.image_size.width as f32)
            .clamp(0., self.max_x_val);
        self.inner.y_val = (self.window_size.height as f32 / self.image_size.height as f32)
            .clamp(0., self.max_y_val);
        self.inner.x_norm = ease::inv_quint_ease_in(self.inner.x_val, 0., self.max_x_val);
        self.inner.y_norm = ease::inv_quint_ease_in(self.inner.y_val, 0., self.max_y_val);

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    /// Adjust the normed horizontal scale.
    pub fn scale_x(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.x_norm = (self.inner.x_norm + amount).clamp(0., 1.);
        self.inner.x_val = ease::quint_ease_in(self.inner.x_norm, 0., self.max_x_val);

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    /// Adjust the normed vertical scale.
    pub fn scale_y(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.y_norm = (self.inner.y_norm + amount).clamp(0., 1.);
        self.inner.y_val = ease::quint_ease_in(self.inner.y_norm, 0., self.max_y_val);

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    /// Keep the image at a constant scale when the window resizes.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>, queue: &wgpu::Queue) {
        let ratio_x = new_size.width as f32 / self.window_size.width as f32;
        let ratio_y = new_size.height as f32 / self.window_size.height as f32;

        self.window_size = new_size;

        self.inner.x_val = (self.inner.x_val * ratio_x).clamp(0., self.max_x_val);
        self.inner.y_val = (self.inner.y_val * ratio_y).clamp(0., self.max_y_val);
        self.inner.x_norm = ease::inv_quint_ease_in(self.inner.x_val, 0., self.max_x_val);
        self.inner.y_norm = ease::inv_quint_ease_in(self.inner.y_val, 0., self.max_y_val);

        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }
}
