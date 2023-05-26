use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::{VirtualKeyCode, WindowEvent},
};

use crate::{
    render::{Layer, LayerMode},
    scale::Scale,
    vertex::{Vertex, SQUARE_VERTICES},
};

#[derive(Debug)]
pub struct AnalysisLayerPass {
    analysis: Vec<Vec<f32>>,
    vertex_buffer: wgpu::Buffer,
    pub scale: Scale,
    pipeline: wgpu::RenderPipeline,
    layer_mode: LayerMode,
    used: bool,
}

impl AnalysisLayerPass {
    pub fn new(
        label: Option<&str>,
        analysis: Vec<Vec<f32>>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        layer_mode: LayerMode,
    ) -> Self {
        let dimensions = PhysicalSize {
            width: analysis.len() as u32,
            height: analysis[0].len() as u32,
        };

        let scale = Scale::new(
            label,
            47.,
            47.,
            (config.width, config.height).into(),
            dimensions,
            device,
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(SQUARE_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let pipeline = Self::create_pipeline(label, config, device, layer_mode);

        let mut pass = AnalysisLayerPass {
            analysis,
            vertex_buffer,
            scale,
            pipeline,
            layer_mode,
            used: false,
        };

        pass.scale.unscale(queue);
        pass
    }

    fn create_pipeline(
        label: Option<&str>,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        layer_mode: LayerMode,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::include_wgsl!("analysis.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
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

impl Layer for AnalysisLayerPass {
    fn resize(&mut self, new_size: PhysicalSize<u32>, queue: &wgpu::Queue) {
        self.scale.resize(new_size, queue);
        if !self.used {
            self.scale.unscale(queue);
        }
    }

    fn handle_event(&mut self, event: &WindowEvent, queue: &wgpu::Queue) {
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
            let mut used = true;

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
                _ => used = false,
            }

            if used {
                self.used = true;
            }
        }
    }

    fn render(&mut self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}
