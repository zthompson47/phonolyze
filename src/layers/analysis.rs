#![warn(missing_docs)]
use wgpu::{util::DeviceExt, PrimitiveTopology};
use winit::{
    dpi::PhysicalSize,
    event::{VirtualKeyCode, WindowEvent},
    window::Window,
};

use crate::{
    render::{Layer, LayerMode},
    scale::Scale,
};

#[allow(dead_code)]
#[derive(Debug)]
pub struct AnalysisLayerPass {
    analysis: Vec<Vec<f32>>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    pub scale: Scale,
    pipeline: wgpu::RenderPipeline,
    layer_mode: LayerMode,
    used: bool,
    vertices: Vec<Vertex>,
    gradient: colorgrad::Gradient,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 4],
    //level: f32,
    color: [f32; 4],
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
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                /*
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<f32>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                */
            ],
        }
    }
}

impl AnalysisLayerPass {
    pub fn new(
        label: Option<&str>,
        analysis: Vec<Vec<f32>>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        layer_mode: LayerMode,
        gradient: colorgrad::Gradient,
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

        let mut vertices = Vec::new();
        let mut indices: Vec<u32> = vec![];

        //let w = 1276;
        //let h = 772;

        dbg!(&dimensions);

        //let w = dimensions.width as usize;
        let w = 1276.clamp(0, dimensions.width as usize); //180;

        let h = dimensions.height as usize;
        //let h = 3000.clamp(0, dimensions.height as usize); //400;

        dbg!(&w, &h);
        analysis.iter().take(w).enumerate().for_each(|(i, x)| {
            x.iter().take(h).enumerate().for_each(|(j, y)| {
                let color = gradient.at(*y as f64).to_array().map(|x| x as f32);
                //color[3] = 0.8 + *y * 0.2;

                vertices.push(Vertex {
                    position: [
                        (i as f32 / (w as f32 - 1.)) * 2. - 1., // - 0.5,
                        (j as f32 / (h as f32 - 1.)) * 2. - 1., // - 0.5,
                        0.,
                        0.,
                    ],
                    //level,
                    color,
                    /*color: if i % 2 == 0 {
                        if j % 2 == 0 {
                            [1., 0., 0., 1.]
                        } else {
                            [0., 0., 1., 1.]
                        }
                    } else if j % 2 == 0 {
                        [0., 1., 0., 1.]
                    } else {
                        [1., 1., 1., 1.]
                    },*/
                });
                if i < w - 1 && j < h - 1 {
                    let bl = (h * i + j) as u32;
                    let br = bl + h as u32;
                    let tl = bl + 1;
                    let tr = br + 1;

                    indices.extend_from_slice(&[[tl, bl, tr], [tr, bl, br]].concat());
                }
            })
        });

        //dbg!(&vertices);
        //dbg!(&indices);

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

        let shader = device.create_shader_module(wgpu::include_wgsl!("analysis.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label,
            bind_group_layouts: &[],
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

        let mut pass = AnalysisLayerPass {
            analysis,
            vertex_buffer,
            index_buffer,
            num_indices,
            scale,
            pipeline,
            layer_mode,
            used: false,
            vertices,
            gradient,
        };

        pass.scale.unscale(queue);

        pass
    }
}

impl Layer for AnalysisLayerPass {
    fn resize(&mut self, new_size: PhysicalSize<u32>, queue: &wgpu::Queue) {
        self.scale.resize(new_size, queue);
        if !self.used {
            self.scale.unscale(queue);
        }
    }

    fn handle_event(&mut self, event: &WindowEvent, queue: &wgpu::Queue) -> bool {
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

        false
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        _window: &Window,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _config: &wgpu::SurfaceConfiguration,
    ) {
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
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        //render_pass.draw(0..self.vertices.len() as u32, 0..1);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}
