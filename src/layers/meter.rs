#![allow(dead_code, unused_imports)]
use std::sync::{mpsc, Arc, Mutex};

use instant::{Duration, Instant};
use wgpu::{util::DeviceExt, PrimitiveTopology};
use winit::{
    dpi::PhysicalSize,
    event::{VirtualKeyCode, WindowEvent},
    window::Window,
};

use crate::{
    audio::{AudioPlayer, PlaybackPosition},
    camera::{Camera, InnerCamera},
    gradient::{Gradient, InnerGradient},
    layers::{Layer, LayerMode},
    render::Renderer,
    uniforms::{self, Progress},
};

use super::LayerState;

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 4],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
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

#[allow(dead_code)]
#[derive(Debug)]
pub struct MeterPass {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    last_update: Instant,
    progress: Progress,
}

impl MeterPass {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        analysis: &[Vec<f32>],
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let label = Some("Meter");
        let (vertex_buffer, index_buffer, num_indices) = tessellate(analysis, device);
        let shader = device.create_shader_module(wgpu::include_wgsl!("meter.wgsl"));
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vertex_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fragment_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
        let progress = uniforms::Progress::new(device);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: progress.binding_resource(),
            }],
            label,
        });

        MeterPass {
            vertex_buffer,
            index_buffer,
            num_indices,
            pipeline,
            bind_group,
            last_update: Instant::now(),
            progress,
        }
    }
}

impl Layer for MeterPass {
    fn resize(&mut self, _new_size: PhysicalSize<u32>, _queue: &wgpu::Queue) {}

    fn handle_event(
        &mut self,
        event: &WindowEvent,
        _queue: &wgpu::Queue,
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
                Some(VirtualKeyCode::Space) => {}
                Some(VirtualKeyCode::Right) => {}
                Some(VirtualKeyCode::Left) => {}
                _ => {}
            }
        }

        egui_winit::EventResponse {
            consumed: false,
            repaint: true,
        }
    }

    fn update(
        &mut self,
        _delta: instant::Duration,
        state: &mut LayerState,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: &Window,
    ) {
        if let Some(progress) = &state.progress {
            if let Ok(progress) = progress.lock() {
                let now = Instant::now();
                let diff = if now > progress.instant {
                    (now - progress.instant).as_secs_f64()
                } else {
                    -(progress.instant - now).as_secs_f64()
                };

                let pos = progress.music_position + diff;

                //if Instant::now().duration_since(self.last_update) > Duration::from_millis(200) {
                self.progress.update_position(pos as f32, queue);
                window.request_redraw();
                self.last_update = Instant::now();
                //}
            }
        }
    }

    fn render(&mut self, renderer: &mut Renderer, _state: &mut LayerState) {
        let mut render_pass = renderer
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Meter"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: renderer.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        store: true,
                        load: wgpu::LoadOp::Load,
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

fn tessellate(_analysis: &[Vec<f32>], device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer, u32) {
    let vertices: Vec<Vertex> = vec![
        Vertex {
            position: [-1.0, 1.0, 0.0, 0.0],
        },
        Vertex {
            position: [3.0, 1.0, 0.0, 0.0],
        },
        Vertex {
            position: [-1.0, -3.0, 0.0, 0.0],
        },
    ];
    let indices: Vec<u32> = vec![0, 1, 2];
    let label = Some("Meter");
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
