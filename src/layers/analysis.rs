//#![deny(elided_lifetimes_in_paths)]
use std::num::NonZeroU32;

use instant::Instant;
use wgpu::{util::DeviceExt, PrimitiveTopology};
use winit::{
    dpi::PhysicalSize,
    event::{VirtualKeyCode, WindowEvent},
    window::Window,
};

use crate::{
    render::{RenderView, Renderer},
    uniforms::Camera,
    uniforms::Gradient,
};

use super::{Layer, LayerMode, LayerState};

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 4],
}

impl<'a> Vertex {
    fn buffer_layout() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x4,
            }],
        }
    }
}

#[derive(Debug)]
pub struct AnalysisLayerPass {
    layer_mode: LayerMode,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    gradient: Gradient,
    camera: Camera,
    last_update: Instant,
    used: bool,
}

impl AnalysisLayerPass {
    pub fn new(
        analysis: &Vec<Vec<f32>>,
        ctx: &RenderView,
        layer_mode: LayerMode,
        gradient: Gradient,
    ) -> Self {
        let label = Some("AnalysisPass");
        let (vertex_buffer, index_buffer, num_indices) = tessellate(analysis, &ctx.device);
        let shader = ctx
            .device
            .create_shader_module(wgpu::include_wgsl!("analysis.wgsl"));
        let bind_group_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label,
                    entries: &[
                        Gradient::bind_group_entry(0),
                        Camera::bind_group_entry(1),
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::D1,
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 3,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 4,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D1,
                                multisampled: false,
                            },
                            count: NonZeroU32::new(4),
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 5,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: NonZeroU32::new(4),
                        },
                    ],
                });
        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });
        let pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label,
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vertex_main",
                    buffers: &[Vertex::buffer_layout()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fragment_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.config.format,
                        blend: Some(match layer_mode {
                            LayerMode::Background => wgpu::BlendState::REPLACE,
                            LayerMode::AlphaBlend => wgpu::BlendState::ALPHA_BLENDING,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });
        let camera = Camera::new(&ctx.device);

        // Color map lut
        let size = wgpu::Extent3d {
            width: 256,
            height: 1,
            depth_or_array_layers: 1,
        };
        let texture_descriptor = wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let gray_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Gray"),
            view_formats: &[],
            ..texture_descriptor
        });
        let red_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Red"),
            view_formats: &[],
            ..texture_descriptor
        });
        let green_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Green"),
            view_formats: &[],
            ..texture_descriptor
        });
        let blue_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Blue"),
            view_formats: &[],
            ..texture_descriptor
        });

        let gray_texture_view = gray_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let red_texture_view = red_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let green_texture_view = green_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let blue_texture_view = blue_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let gray_texture_data: Vec<u8> = (0..=255).flat_map(|x| [x, x, x, 255]).collect();
        let red_texture_data: Vec<u8> = (0..=255).flat_map(|x| [x, 0, 0, 255]).collect();
        let green_texture_data: Vec<u8> = (0..=255).flat_map(|x| [0, x, 0, 255]).collect();
        let blue_texture_data: Vec<u8> = (0..=255).flat_map(|x| [255 - x, 0, x, 255]).collect();

        ctx.queue.write_texture(
            gray_texture.as_image_copy(),
            &gray_texture_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(1024),
                rows_per_image: None,
            },
            size,
        );
        ctx.queue.write_texture(
            red_texture.as_image_copy(),
            &red_texture_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(1024),
                rows_per_image: None,
            },
            size,
        );
        ctx.queue.write_texture(
            green_texture.as_image_copy(),
            &green_texture_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(1024),
                rows_per_image: None,
            },
            size,
        );
        ctx.queue.write_texture(
            blue_texture.as_image_copy(),
            &blue_texture_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(1024),
                rows_per_image: None,
            },
            size,
        );

        let texture = ctx.device.create_texture(&texture_descriptor);
        let img: Vec<u8> = (0..=255).flat_map(|x| [x, x, x, 255]).collect();
        ctx.queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &img,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(1024),
                rows_per_image: None,
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D1),
            ..Default::default()
        });

        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let gray_sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let red_sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let green_sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let blue_sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gradient.binding_resource(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera.binding_resource(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },

                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureViewArray(&[
                        &gray_texture_view,
                        &red_texture_view,
                        &green_texture_view,
                        &blue_texture_view,
                    ]),
                },

                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::SamplerArray(&[
                        &gray_sampler,
                        &red_sampler,
                        &green_sampler,
                        &blue_sampler,
                    ]),
                },

            ],
        });

        AnalysisLayerPass {
            vertex_buffer,
            index_buffer,
            num_indices,
            pipeline,
            layer_mode,
            gradient,
            bind_group,
            camera,
            last_update: Instant::now(),
            used: false,
        }
    }
}

fn tessellate(
    analysis: &Vec<Vec<f32>>,
    device: &wgpu::Device,
) -> (wgpu::Buffer, wgpu::Buffer, u32) {
    let mut vertices = Vec::new();
    let mut indices: Vec<u32> = vec![];
    let width = analysis.len();
    let height = analysis[0].len();

    analysis
        .iter()
        .take(width)
        .enumerate()
        .for_each(|(i, col)| {
            col.iter().take(height).enumerate().for_each(|(j, level)| {
                // Normalize for the shader.
                let level = ((level + 180.0) / 190.0).clamp(0.0, 1.0);
                let vertex = Vertex {
                    position: [
                        (i as f32 / (width as f32 - 1.0)),
                        (j as f32 / (height as f32 - 1.0)),
                        level,
                        0.0,
                    ],
                };

                vertices.push(vertex);

                if i < width - 1 && j < height - 1 {
                    let bottom_left = (height * i + j) as u32;
                    let bottom_right = bottom_left + height as u32;
                    let top_left = bottom_left + 1;
                    let top_right = bottom_right + 1;

                    indices.extend_from_slice(
                        &[
                            [top_left, bottom_left, top_right],
                            [top_right, bottom_left, bottom_right],
                        ]
                        .concat(),
                    );
                }
            })
        });

    let label = Some("Update Analysis");
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label,
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label,
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    let num_indices = indices.len() as u32;

    (vertex_buffer, index_buffer, num_indices)
}

impl Layer for AnalysisLayerPass {
    fn resize(
        &mut self,
        _new_size: PhysicalSize<u32>,
        _queue: &wgpu::Queue,
        _state: &mut LayerState,
    ) {
    }

    fn handle_event(
        &mut self,
        event: &WindowEvent,
        queue: &wgpu::Queue,
        state: &mut LayerState,
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
                    if state.modifiers.shift() {
                        self.camera.move_x(-0.03, queue);
                    } else {
                        //scale.scale_x(-0.01, queue);
                        self.camera.scale_x(-0.01, queue);
                    }
                }

                Some(VirtualKeyCode::Right | VirtualKeyCode::L) => {
                    if state.modifiers.shift() {
                        self.camera.move_x(0.03, queue);
                    } else {
                        //scale.scale_x(0.01, queue);
                        self.camera.scale_x(0.01, queue);
                    }
                }

                Some(VirtualKeyCode::Down | VirtualKeyCode::J) => {
                    if state.modifiers.shift() {
                        self.camera.move_y(-0.03, queue);
                    } else {
                        //scale.scale_y(-0.01, queue);
                        self.camera.scale_y(-0.01, queue);
                    }
                }

                Some(VirtualKeyCode::Up | VirtualKeyCode::K) => {
                    if state.modifiers.shift() {
                        self.camera.move_y(0.03, queue);
                    } else {
                        //scale.scale_y(0.01, queue);
                        self.camera.scale_y(0.01, queue);
                    }
                }

                Some(VirtualKeyCode::M) if state.modifiers.logo() => {
                    self.camera.zero(queue);
                }

                Some(VirtualKeyCode::N) if state.modifiers.logo() => {
                    self.camera.fill(queue);
                }

                _ => {
                    dbg!(virtual_keycode);
                    return egui_winit::EventResponse {
                        consumed: false,
                        repaint: false,
                    };
                }
            }
            //dbg!(&scale.window_size, &scale.image_size, &scale.inner);
            self.used = true;
            return egui_winit::EventResponse {
                consumed: false,
                repaint: true,
            };
            //}
        }

        egui_winit::EventResponse {
            consumed: false,
            repaint: false,
        }
    }

    fn update(
        &mut self,
        _delta: instant::Duration,
        state: &mut LayerState,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        _window: &Window,
    ) {
        if let Some(new_color_map) = state.update_color_map() {
            self.gradient.update(new_color_map.uniform(), queue);
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
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
