//use easer::functions::Easing;
use image::{DynamicImage, GenericImageView};
use wgpu::util::DeviceExt;
use winit::{
    dpi::PhysicalSize,
    event::{VirtualKeyCode, WindowEvent},
};

use crate::{
    render::{Layer, LayerMode},
    vertex::{Vertex, SQUARE_VERTICES},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct InnerScale {
    x_norm: f32,
    y_norm: f32,
    x_val: f32,
    y_val: f32,
}

#[derive(Debug)]
pub struct Scale {
    window_size: PhysicalSize<u32>,
    image_size: PhysicalSize<u32>,
    inner: InnerScale,
    buffer: wgpu::Buffer,
}

impl Scale {
    const MAX_X_SCALE: f32 = 4.;
    const MAX_Y_SCALE: f32 = 2.;

    fn _unscale(&mut self) {}

    fn center(&mut self, queue: &wgpu::Queue) {
        self.inner.x_val = (self.window_size.width as f32 / self.image_size.width as f32)
            .clamp(0., Self::MAX_X_SCALE);
        self.inner.y_val = (self.window_size.height as f32 / self.image_size.height as f32)
            .clamp(0., Self::MAX_Y_SCALE);
        self.inner.x_norm = inv_quint_ease_in(self.inner.x_val, 0., Self::MAX_X_SCALE);
        self.inner.y_norm = inv_quint_ease_in(self.inner.y_val, 0., Self::MAX_Y_SCALE);
        //queue.write_buffer(&self.scale_buffer, 0, bytemuck::cast_slice(&[self.scale]));
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    fn scale_x(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.x_norm = (self.inner.x_norm + amount).clamp(0., 1.);
        self.inner.x_val = quint_ease_in(self.inner.x_norm, 0., Self::MAX_X_SCALE);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    fn scale_y(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.inner.y_norm = (self.inner.y_norm + amount).clamp(0., 1.);
        self.inner.y_val = quint_ease_in(self.inner.y_norm, 0., Self::MAX_Y_SCALE);
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.inner]));
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        log::info!("{:#?}", self);
        self.window_size = new_size;
        log::info!("{:#?}", self);
    }
}

#[derive(Debug)]
pub struct ImageLayerPass {
    //label: String,
    pub image: DynamicImage,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    //scale_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    pub scale: Scale,
    pipeline: wgpu::RenderPipeline,
    layer_mode: LayerMode,
    used: bool,
}

impl ImageLayerPass {
    pub fn new(
        label: Option<&str>,
        image: DynamicImage,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        //blend_state: wgpu::BlendState,
        layer_mode: LayerMode,
    ) -> Self {
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

        let inner = InnerScale {
            x_norm: 1.,
            y_norm: 1.,
            x_val: inv_quint_ease_in(1., 0., Scale::MAX_X_SCALE),
            y_val: inv_quint_ease_in(1., 0., Scale::MAX_Y_SCALE),
        };
        let scale = Scale {
            inner,
            window_size: (config.width, config.height).into(),
            image_size: dimensions.into(),
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label,
                contents: bytemuck::cast_slice(&[inner]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        };
        /*
        let scale_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(&[scale]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        */

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
                    //resource: scale_buffer.as_entire_binding(),
                    resource: scale.buffer.as_entire_binding(),
                },
            ],
            label,
        });

        let pipeline = Self::create_pipeline(label, config, device, &bind_group_layout, layer_mode);

        let mut pass = ImageLayerPass {
            //label: String::from(label.unwrap_or("")),
            image,
            bind_group_layout,
            bind_group,
            //scale_buffer,
            vertex_buffer,
            scale,
            pipeline,
            layer_mode,
            used: false,
        };

        log::info!("{:#?}", &pass.scale);
        pass.scale.center(queue);
        log::info!("{:#?}", &pass.scale);
        pass
    }

    //const MAX_X_SCALE: f32 = 4.;
    //const MAX_Y_SCALE: f32 = 2.;

    /*
    fn scale_x(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.scale.x_norm = (self.scale.x_norm + amount).clamp(0., 1.);
        self.scale.x_val = quint_ease_in(self.scale.x_norm, 0., Self::MAX_X_SCALE);
        queue.write_buffer(&self.scale.buffer, 0, bytemuck::cast_slice(&[self.scale]));
    }

    fn scale_y(&mut self, amount: f32, queue: &wgpu::Queue) {
        self.scale.y_norm = (self.scale.y_norm + amount).clamp(0., 1.);
        self.scale.y_val = quint_ease_in(self.scale.y_norm, 0., Self::MAX_Y_SCALE);
        queue.write_buffer(&self.scale.buffer, 0, bytemuck::cast_slice(&[self.scale]));
    }
    */

    /*
    fn center(&mut self, new_size: PhysicalSize<u32>, queue: &wgpu::Queue) {
        self.scale.x_val =
            (new_size.width as f32 / self.image.dimensions().0 as f32).clamp(0., Self::MAX_X_SCALE);
        self.scale.y_val = (new_size.height as f32 / self.image.dimensions().1 as f32)
            .clamp(0., Self::MAX_Y_SCALE);
        self.scale.x_norm = inv_quint_ease_in(self.scale.x_val, 0., Self::MAX_X_SCALE);
        self.scale.y_norm = inv_quint_ease_in(self.scale.y_val, 0., Self::MAX_Y_SCALE);
        queue.write_buffer(&self.scale.buffer, 0, bytemuck::cast_slice(&[self.scale]));
    }
    */

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

fn quint_ease_in(x: f32, min: f32, max: f32) -> f32 {
    assert!(x >= 0.);
    assert!(x <= 1.0);
    (max - min) * x.powi(5) + min
}

fn inv_quint_ease_in(x: f32, min: f32, max: f32) -> f32 {
    assert!(x >= min);
    assert!(x <= max);
    ((x - min) / (max - min)).powf(1. / 5.)
}

impl Layer for ImageLayerPass {
    fn resize(&mut self, new_size: PhysicalSize<u32>, queue: &wgpu::Queue) {
        let _ = new_size;
        let _ = queue;
        self.scale.resize(new_size);
        if !self.used {
            self.scale.center(queue);
        }
        /*
        self.scale[0] = new_size.width as f32 / self.image.dimensions().0 as f32;
        self.scale[1] = new_size.height as f32 / self.image.dimensions().1 as f32;
        self.scale[2] = new_size.height as f32;
        queue.write_buffer(&self.scale_buffer, 0, bytemuck::cast_slice(&self.scale));
        */
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
                    self.scale.center(queue);
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

        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}

/*
pub struct ShowAnalysisPass {
    pub image: DynamicImage,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    scale_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    pub scale: [f32; 4],
    pipeline: wgpu::RenderPipeline,
}

impl ShowAnalysisPass {
    pub fn new(
        label: Option<&str>,
        image: DynamicImage,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
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
            label,
        });

        let scale = [1.0, 1.0, 0.0, 0.0];
        let scale_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&scale),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
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
                    resource: scale_buffer.as_entire_binding(),
                },
            ],
            label,
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Background"),
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
                    //blend: Some(wgpu::BlendState::REPLACE),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), // !!!!!!!!!!!!!!!!!!!!!!!!1
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
        });

        ShowAnalysisPass {
            image,
            bind_group_layout,
            bind_group,
            scale_buffer,
            vertex_buffer,
            scale,
            pipeline,
        }
    }

    fn update_scale(&mut self, new_scale: [f32; 4], queue: &wgpu::Queue) {
        self.scale
            .iter_mut()
            .zip(new_scale.iter())
            .for_each(|(a, b)| {
                *a += *b;
            });
        queue.write_buffer(&self.scale_buffer, 0, bytemuck::cast_slice(&self.scale));
    }
}

impl Layer for ShowAnalysisPass {
    fn resize(&mut self, new_size: PhysicalSize<u32>, queue: &wgpu::Queue) {
        self.scale[0] = new_size.width as f32 / self.image.dimensions().0 as f32;
        self.scale[1] = new_size.height as f32 / self.image.dimensions().1 as f32;
        self.scale[2] = new_size.height as f32;
        queue.write_buffer(&self.scale_buffer, 0, bytemuck::cast_slice(&self.scale));
    }

    fn handle_event(&mut self, event: &WindowEvent, queue: &wgpu::Queue) {
        if let WindowEvent::KeyboardInput { input, .. } = event {
            match input.virtual_keycode {
                Some(VirtualKeyCode::Left | VirtualKeyCode::H) => {
                    self.update_scale([0.1, 0., 0., 0.], queue)
                }
                Some(VirtualKeyCode::Right | VirtualKeyCode::L) => {
                    self.update_scale([-0.1, 0., 0., 0.], queue)
                }
                Some(VirtualKeyCode::Down | VirtualKeyCode::J) => { // !!!!!!!!!!!!!!2
                    self.update_scale([0., 0.1, 0., 0.], queue)
                }
                Some(VirtualKeyCode::Up | VirtualKeyCode::K) => { // !!!!!!!!!!!!!!2
                    self.update_scale([0., -0.1, 0., 0.], queue)
                }
                _ => {}
            }
        }
    }

    fn render(&mut self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations { // !!!!!!!!!!!!!!3
                    load: wgpu::LoadOp::Load, /*wgpu::LoadOp::Clear(wgpu::Color {
                                                  r: 0.0,
                                                  g: 0.3,
                                                  b: 0.5,
                                                  a: 1.0,
                                              })*/
                    store: true,
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
*/
