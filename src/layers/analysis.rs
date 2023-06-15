#![allow(unused_imports)]
use wgpu::{util::DeviceExt, PrimitiveTopology};
use winit::{
    dpi::PhysicalSize,
    event::{VirtualKeyCode, WindowEvent},
};

use crate::{
    gradient::{Gradient, InnerGradient},
    layers::{Layer, LayerMode},
    render::Renderer,
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
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x4,
                },
                /*
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                */
            ],
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct AnalysisLayerPass {
    analysis: Vec<Vec<f32>>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    pipeline: wgpu::RenderPipeline,
    layer_mode: LayerMode,
    used: bool,
    gradient: Gradient,
    bind_group: wgpu::BindGroup,
}

impl AnalysisLayerPass {
    pub fn new(
        label: Option<&str>,
        analysis: Vec<Vec<f32>>,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        layer_mode: LayerMode,
        gradient: Gradient,
    ) -> Self {
        let _dimensions = PhysicalSize {
            width: analysis.len() as u32,
            height: analysis[0].len() as u32,
        };
        let (vertex_buffer, index_buffer, num_indices) = update_analysis(&analysis, device);
        let shader = device.create_shader_module(wgpu::include_wgsl!("analysis.wgsl"));
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: gradient.binding_resource(),
            }],
            label,
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
                //topology: PrimitiveTopology::PointList,
                topology: PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        AnalysisLayerPass {
            analysis,
            vertex_buffer,
            index_buffer,
            num_indices,
            pipeline,
            layer_mode,
            used: false,
            gradient,
            bind_group,
        }
    }
}

fn update_analysis(
    analysis: &Vec<Vec<f32>>,
    device: &wgpu::Device,
) -> (wgpu::Buffer, wgpu::Buffer, u32) {
    let mut vertices = Vec::new();
    let mut indices: Vec<u32> = vec![];
    //let w = 1276;
    //let h = 772;
    //let w = dimensions.width as usize;
    //let w = 1276.clamp(0, dimensions.width as usize); //180;
    //let h = dimensions.height as usize;
    //let h = 3000.clamp(0, dimensions.height as usize); //400;

    let dimensions = PhysicalSize {
        width: analysis.len() as u32,
        height: analysis[0].len() as u32,
    };

    let w = dimensions.width as usize;
    let h = dimensions.height as usize;

    analysis.iter().take(w).enumerate().for_each(|(i, x)| {
        x.iter().take(h).enumerate().for_each(|(j, y)| {
            let level = *y;
            //let color = gradient.at(level as f64).to_array().map(|x| x as f32);
            //dbg!(*y);

            //let mut level = (*y + 150.0).clamp(0.0, 150.0);
            //level /= 150.0;

            //dbg!(&level);

            let vertex = Vertex {
                position: [
                    (i as f32 / (w as f32 - 1.0)) * 2.0 - 1.0,
                    (j as f32 / (h as f32 - 1.0)) * 2.0 - 1.0,
                    //*y,
                    level,
                    0.0,
                ],
                //level: *y,
                //level,
                //color,
                //..Default::default()
            };
            //dbg!(&vertex);
            vertices.push(vertex);

            if i < w - 1 && j < h - 1 {
                let bl = (h * i + j) as u32;
                let br = bl + h as u32;
                let tl = bl + 1;
                let tr = br + 1;

                indices.extend_from_slice(&[[tl, bl, tr], [tr, bl, br]].concat());
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
    fn resize(&mut self, _new_size: PhysicalSize<u32>, _queue: &wgpu::Queue) {}

    fn handle_event(
        &mut self,
        _event: &WindowEvent,
        _queue: &wgpu::Queue,
    ) -> egui_winit::EventResponse {
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
    ) {
        if let Some(new_color_map) = state.update_color_map() {
            self.gradient.update(new_color_map.grad(), queue);
        }
    }

    fn render(&mut self, renderer: &mut Renderer) {
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
