use colorgrad::{Color, CustomGradient};
use strum_macros::{Display, EnumIter};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InnerGradient {
    pub index: [u32; 4],
}

#[derive(Debug)]
pub struct Gradient {
    inner: InnerGradient,
    buffer: wgpu::Buffer,
    color_maps: Vec<ColorMap>,
    textures: Vec<wgpu::TextureView>,
}

impl Gradient {
    pub fn new(
        label: Option<&str>,
        inner: InnerGradient,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let color_maps = vec![
            ColorMap::Blue,
            ColorMap::Rgb,
            ColorMap::RgbInv,
            ColorMap::Crazy,
        ];

        let textures = color_maps
            .iter()
            .map(|color_map| {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label,
                    ..ColorMap::texture_descriptor()
                });
                let data: Vec<u8> = color_map.data();
                let size = wgpu::Extent3d {
                    width: 256,
                    height: 1,
                    depth_or_array_layers: 1,
                };

                queue.write_texture(
                    texture.as_image_copy(),
                    &data,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(1024),
                        rows_per_image: None,
                    },
                    size,
                );

                texture.create_view(&wgpu::TextureViewDescriptor::default())
            })
            .collect();

        Gradient {
            inner,
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(&[inner]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
            color_maps,
            textures,
        }
    }

    pub fn update(&mut self, inner: InnerGradient, queue: &wgpu::Queue) {
        self.inner = inner;
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    pub fn bind_group_entry(index: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: index,
            visibility: wgpu::ShaderStages::FRAGMENT,
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

#[derive(Copy, Clone, Debug, Default, EnumIter, Display, PartialEq)]
pub enum ColorMap {
    Blue,
    Gray,
    Green,
    Red,
    #[default]
    Rgb,
    RgbInv,
    Crazy,
    Zonks,
}

impl ColorMap {
    fn texture_descriptor() -> wgpu::TextureDescriptor<'static> {
        let size = wgpu::Extent3d {
            width: 256,
            height: 1,
            depth_or_array_layers: 1,
        };

        wgpu::TextureDescriptor {
            label: Some("ColorMap"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        }
    }

    /*
    fn create_buffer(&self, device: &wgpu::Device) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("--text--"),
            ..Self::texture_descriptor()
        });
    }
    */

    fn data(&self) -> Vec<u8> {
        match &self {
            //r: [0.0, 0.0, 0.0, 1.0],
            //g: [0.0, 0.0, 1.0, 0.0],
            //b: [0.0, 1.0, 0.0, 0.0],
            //a: [0.0, 0.8, 1.0, 1.0],
            //domain: [-150.0, -80.0, -40.0, 0.0],
            Self::Rgb => {
                //(0..=255).flat_map(|x| [x, x, x, 255]).collect(),

                colorgrad::CustomGradient::new()
                    .colors(&[
                        Color::from_rgba8(0, 0, 0, 0),
                        Color::from_rgba8(0, 0, 255, 204),
                        Color::from_rgba8(0, 255, 0, 255),
                        Color::from_rgba8(255, 0, 0, 255),
                    ])
                    .build()
                    .unwrap()
                    .colors(256)
                    .iter()
                    .flat_map(|c| c.to_rgba8())
                    .collect()
            }

            //r: [0.0, 0.0, 0.0, 0.2],
            //g: [0.0, 0.0, 0.2, 0.2],
            //b: [0.0, 1.0, 0.5, 1.0],
            //a: [0.0, 0.8, 1.0, 1.0],
            //domain: [-150.0, -80.0, -40.0, 0.0],
            Self::Blue => (0..=255).flat_map(|x| [x, x, x, 255]).collect(),

            //r: [1.0, 0.0, 0.0, 0.0],
            //g: [0.0, 1.0, 0.0, 0.0],
            //b: [0.0, 0.0, 1.0, 0.0],
            //a: [1.0, 1.0, 0.8, 0.0],
            //domain: [-150.0, -80.0, -40.0, 0.0],
            Self::RgbInv => (0..=255).flat_map(|x| [x, x, x, 255]).collect(),

            //r: [1.0, 0.2, 0.8, 0.2],
            //g: [0.0, 1.0, 0.0, 0.5],
            //b: [0.2, 0.0, 0.7, 0.3],
            //a: [1.0, 1.0, 0.8, 0.0],
            //domain: [-150.0, -100.0, -80.0, 0.0],
            Self::Crazy => (0..=255).flat_map(|x| [x, x, x, 255]).collect(),

            Self::Gray => (0..=255).flat_map(|x| [x, x, x, 255]).collect(),
            Self::Zonks => (0..=255).flat_map(|x| [255 - x, 0, x, 255]).collect(),
            _ => (0..=255).flat_map(|x| [x, x, x, 255]).collect(),
        }
    }

    pub fn uniform(&self) -> InnerGradient {
        match &self {
            Self::Rgb => InnerGradient {
                index: [0, 0, 0, 0],
            },
            Self::Blue => InnerGradient {
                index: [1, 0, 0, 0],
            },
            Self::RgbInv => InnerGradient {
                index: [2, 0, 0, 0],
            },
            Self::Crazy => InnerGradient {
                index: [3, 0, 0, 0],
            },
            _ => InnerGradient {
                index: [4, 0, 0, 0],
            },
        }
    }
}
