use wgpu::{util::DeviceExt, PrimitiveTopology};
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    window::Window, keyboard::{Key, NamedKey},
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
                        gradient.texture_bg_entry(2),
                        gradient.sampler_bg_entry(3),
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
                    resource: wgpu::BindingResource::TextureView(&gradient.texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&gradient.sampler),
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
                use crate::uniforms::NormDb;
                let vertex = Vertex {
                    position: [
                        (i as f32 / (width as f32 - 1.0)),
                        (j as f32 / (height as f32 - 1.0)),
                        level.normalize_decibels(),
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
        _window: &winit::window::Window,
    ) -> egui_winit::EventResponse {
        if let WindowEvent::KeyboardInput { event, .. } = event {
            match event.logical_key.as_ref() {
                Key::Named(NamedKey::ArrowLeft) | Key::Character("H") => {
                    if state.modifiers.shift_key() {
                        self.camera.move_x(-0.03, queue);
                    } else {
                        self.camera.scale_x(-0.01, queue);
                    }
                }

                Key::Named(NamedKey::ArrowRight) | Key::Character("L") => {
                    if state.modifiers.shift_key() {
                        self.camera.move_x(0.03, queue);
                    } else {
                        self.camera.scale_x(0.01, queue);
                    }
                }

                Key::Named(NamedKey::ArrowDown) | Key::Character("J") => {
                    if state.modifiers.shift_key() {
                        self.camera.move_y(-0.03, queue);
                    } else {
                        self.camera.scale_y(-0.01, queue);
                    }
                }

                Key::Named(NamedKey::ArrowUp) | Key::Character("K") => {
                    if state.modifiers.shift_key() {
                        self.camera.move_y(0.03, queue);
                    } else {
                        self.camera.scale_y(0.01, queue);
                    }
                }

                Key::Character("M") if state.modifiers.super_key() => {
                    self.camera.zero(queue);
                }

                Key::Character("N") if state.modifiers.super_key() => {
                    self.camera.fill(queue);
                }

                _ => {
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
            self.gradient
                .update_gradient_texture(new_color_map.data(), queue);
        }
    }

    fn render(&mut self, renderer: &mut Renderer, _state: &mut LayerState) {
        let mut render_pass = renderer
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                occlusion_query_set: None,
                timestamp_writes: None,
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: renderer.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        store: wgpu::StoreOp::Store,
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
