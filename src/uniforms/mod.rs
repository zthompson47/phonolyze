mod camera;
mod gradient;
mod progress;
mod scale;

pub use camera::Camera;
pub use gradient::{ColorMap, Gradient, InnerGradient};
pub use progress::{InnerProgress, Progress};
pub use scale::Scale;

/*trait Uniform {
    fn bind_group_entry(
        binding: u32,
        visibility: wgpu::ShaderStages,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn binding_resource(&self) -> wgpu::BindingResource {
        self.buffer.as_entire_binding()
    }
}*/
