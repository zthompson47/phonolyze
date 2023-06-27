use image::{DynamicImage, GenericImageView};
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::{VirtualKeyCode, WindowEvent},
};

use crate::{
    layers::{Layer, LayerMode},
    render::Renderer,
    uniforms::Scale,
};

use super::LayerState;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl<'a> Vertex {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub const SQUARE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1., 1., 0.],
        tex_coords: [0., 1.],
    },
    Vertex {
        position: [-1., -1., 0.],
        tex_coords: [0., 0.],
    },
    Vertex {
        position: [1., 1., 0.],
        tex_coords: [1., 1.],
    },
    Vertex {
        position: [1., 1., 0.],
        tex_coords: [1., 1.],
    },
    Vertex {
        position: [-1., -1., 0.],
        tex_coords: [0., 0.],
    },
    Vertex {
        position: [1., -1., 0.],
        tex_coords: [1., 0.],
    },
];

#[derive(Debug)]
pub struct ScaledImagePass {
    pub image: DynamicImage,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    pub scale: Scale,
    pipeline: wgpu::RenderPipeline,
    layer_mode: LayerMode,
    used: bool,
}

impl ScaledImagePass {
    pub fn new(
        image: DynamicImage,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        layer_mode: LayerMode,
    ) -> Self {
        let label = Some("ScaledImagePass");
        let dimensions = image.dimensions();
        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let rgba = image.to_rgba8();

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let scale = Scale::new(
            47.,
            47.,
            (config.width, config.height).into(),
            dimensions.into(),
            device,
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(SQUARE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: scale.binding_resource(),
                },
            ],
            label,
        });

        let pipeline = Self::create_pipeline(label, config, device, &bind_group_layout, layer_mode);

        let mut pass = ScaledImagePass {
            image,
            bind_group_layout,
            bind_group,
            vertex_buffer,
            scale,
            pipeline,
            layer_mode,
            used: false,
        };

        log::info!("{:#?}", &pass.scale);
        pass.scale.unscale(queue);
        log::info!("{:#?}", &pass.scale);
        pass
    }

    fn create_pipeline(
        label: Option<&str>,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        layer_mode: LayerMode,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label,
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(match layer_mode {
                        LayerMode::Background => wgpu::BlendState::REPLACE,
                        LayerMode::AlphaBlend => wgpu::BlendState::ALPHA_BLENDING,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Cw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }
}

impl Layer for ScaledImagePass {
    fn resize(
        &mut self,
        new_size: PhysicalSize<u32>,
        queue: &wgpu::Queue,
        _state: &mut LayerState,
    ) {
        self.scale.resize(new_size, queue);
        if !self.used {
            self.scale.unscale(queue);
        }
    }

    fn handle_event(
        &mut self,
        event: &WindowEvent,
        queue: &wgpu::Queue,
        _state: &mut LayerState,
    ) -> egui_winit::EventResponse {
        if let WindowEvent::KeyboardInput {
            input:
                winit::event::KeyboardInput {
                    virtual_keycode,
                    state: winit::event::ElementState::Pressed,
                    ..
                },
            ..
        } = event
        {
            match virtual_keycode {
                Some(VirtualKeyCode::Left | VirtualKeyCode::H) => {
                    self.scale.scale_x(0.01, queue);
                }
                Some(VirtualKeyCode::Right | VirtualKeyCode::L) => {
                    self.scale.scale_x(-0.01, queue);
                }
                Some(VirtualKeyCode::Down | VirtualKeyCode::J) => {
                    self.scale.scale_y(0.01, queue);
                }
                Some(VirtualKeyCode::Up | VirtualKeyCode::K) => {
                    self.scale.scale_y(-0.01, queue);
                }
                Some(VirtualKeyCode::F) => {
                    self.scale.unscale(queue);
                }
                _ => {
                    return egui_winit::EventResponse {
                        consumed: false,
                        repaint: false,
                    }
                }
            }
            self.used = true;
        }

        egui_winit::EventResponse {
            consumed: false,
            repaint: false,
        }
    }

    fn render(&mut self, renderer: &mut Renderer, _state: &mut LayerState) {
        let mut render_pass = renderer
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: renderer.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        store: true,
                        load: match self.layer_mode {
                            LayerMode::AlphaBlend => wgpu::LoadOp::Load,
                            LayerMode::Background => wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                        },
                    },
                })],
                depth_stencil_attachment: None,
            });

        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}
