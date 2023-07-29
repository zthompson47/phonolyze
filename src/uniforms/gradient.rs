use colorgrad::Color;
use strum::IntoEnumIterator;
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
    textures: Vec<wgpu::TextureView>,
}

impl Gradient {
    pub fn texture_count(&self) -> u32 {
        self.textures.len() as u32
    }

    pub fn texture_views(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Vec<wgpu::TextureView> {
        ColorMap::iter()
            .map(|color_map| {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(format!("{:?}", color_map).as_str()),
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
            .collect()
    }

    pub fn samplers(&self, device: &wgpu::Device) -> Vec<wgpu::Sampler> {
        let mut result = Vec::new();

        for _ in 0..self.textures.len() {
            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

            result.push(sampler);
        }

        result
    }

    pub fn new(
        label: Option<&str>,
        inner: InnerGradient,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let textures = ColorMap::iter()
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
            //color_maps,
            textures,
            //sampler,
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

    fn data(&self) -> Vec<u8> {
        match &self {
            Self::Rgb => grad([
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.8],
                [0.0, 1.0, 0.0, 1.0],
                [1.0, 0.0, 0.0, 1.0],
            ]),
            Self::Blue => grad([
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.8],
                [0.0, 0.2, 0.5, 1.0],
                [0.2, 0.2, 1.0, 1.0],
            ]),
            Self::Green => grad([
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.3, 0.0, 0.8],
                [0.0, 0.6, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
            ]),
            Self::Red => grad([
                [0.0, 0.0, 0.0, 0.0],
                [0.3, 0.0, 0.0, 0.8],
                [0.6, 0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0, 1.0],
            ]),
            Self::RgbInv => grad([
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0, 0.8],
                [0.0, 0.0, 0.0, 0.0],
            ]),
            Self::Crazy => grad([
                [1.0, 0.0, 0.2, 1.0],
                [0.2, 1.0, 0.0, 1.0],
                [0.8, 0.0, 0.7, 0.8],
                [0.2, 0.5, 0.3, 0.0],
            ]),
            Self::Gray => (0..=255).flat_map(|x| [x, x, x, 255]).collect(),
            Self::Zonks => (0..=255).flat_map(|x| [255 - x, 0, x, 255]).collect(),
        }
    }

    pub fn uniform(&self) -> InnerGradient {
        for (i, c) in Self::iter().enumerate() {
            if c == *self {
                return InnerGradient {
                    index: [i as u32, 0, 0, 0],
                };
            }
        }
        panic!()
    }
}

pub trait NormDb<T> {
    fn normalize_decibels(&self) -> T;
}

impl NormDb<f32> for f32 {
    fn normalize_decibels(&self) -> f32 {
        ((*self + 150.0) / 150.0).clamp(0.0, 1.0)
    }
}

fn grad(mat: [[f64; 4]; 4]) -> Vec<u8> {
    colorgrad::CustomGradient::new()
        .colors(&[
            Color::from_linear_rgba(mat[0][0], mat[0][1], mat[0][2], mat[0][3]),
            Color::from_linear_rgba(mat[1][0], mat[1][1], mat[1][2], mat[1][3]),
            Color::from_linear_rgba(mat[2][0], mat[2][1], mat[2][2], mat[2][3]),
            Color::from_linear_rgba(mat[3][0], mat[3][1], mat[3][2], mat[3][3]),
        ])
        .domain(&[0.0, 0.467, 0.733, 1.0])
        .build()
        .unwrap()
        .colors(256)
        .iter()
        .flat_map(|c| c.to_rgba8())
        .collect()
}
