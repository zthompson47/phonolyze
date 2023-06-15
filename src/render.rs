use winit::{dpi::PhysicalSize, window::Window};

use crate::layers::{Layer, LayerState};

pub struct Renderer<'a> {
    pub view: &'a wgpu::TextureView,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub window: &'a Window,
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub config: &'a wgpu::SurfaceConfiguration,
    pub state: &'a mut LayerState,
}

pub struct RenderView {
    size: PhysicalSize<u32>,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub layers: Vec<Box<dyn Layer>>,
    pub layer_state: LayerState,
    pub scale_factor: f32,
}

impl RenderView {
    pub async fn new(window: &winit::window::Window) -> Self {
        let scale_factor = window.scale_factor() as f32;
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..wgpu::InstanceDescriptor::default()
        });
        // SAFETY: `View` is created in the main thread and `window` remains valid
        // for the lifetime of `surface`.
        let surface = unsafe { instance.create_surface(&window).unwrap() };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let limits = wgpu::Limits::downlevel_webgl2_defaults();
        let _max_width = limits.max_texture_dimension_2d;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits,
                },
                None,
            )
            .await
            .unwrap();
        let capabilities = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: capabilities.formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![capabilities.formats[0]],
        };

        dbg!(config.usage, config.format, size.width);

        surface.configure(&device, &config);

        RenderView {
            size,
            surface,
            device,
            queue,
            config,
            layers: vec![],
            layer_state: LayerState::default(),
            scale_factor,
        }
    }

    /*
    pub async fn _update_background(&mut self, filename: &str) {
        let background_image = load_image(filename).await;
        let _background = ScaledImagePass::new(
            Some("Background Image"),
            background_image,
            &self.device,
            &self.queue,
            &self.config,
            wgpu::BlendState::REPLACE,
        );
    }
    */

    pub fn update(&mut self, delta: instant::Duration) {
        let _step = delta.as_secs_f32();

        self.layers.iter_mut().for_each(|layer| {
            layer.update(delta, &mut self.layer_state, &self.device, &self.queue);
        });

        /*
        let shader = self
            .device
            .create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        self.pipeline.vertex.module = &shader;
        */
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.layers.iter_mut().for_each(|layer| {
                layer.resize(new_size, &self.queue);
            });
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, window: &Window) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut renderer = Renderer {
            view: &view,
            encoder: &mut encoder,
            window,
            device: &self.device,
            queue: &self.queue,
            config: &self.config,
            state: &mut self.layer_state,
        };

        self.layers.iter_mut().for_each(|layer| {
            layer.render(&mut renderer);
        });

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }

    pub fn capture_layer<F>(&mut self, f: F)
    where
        F: FnOnce(&wgpu::Device, &wgpu::Queue, &wgpu::SurfaceConfiguration, f32) -> Box<dyn Layer>,
    {
        self.layers.push(f(
            &self.device,
            &self.queue,
            &self.config,
            self.scale_factor,
        ));
    }
}
