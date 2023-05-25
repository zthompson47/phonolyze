use image::{Rgba, RgbaImage};
use winit::{dpi::PhysicalSize, event::WindowEvent};

use crate::{audio::AudioFile, fft::stft, file::load_image, texture::ImageLayerPass, Cli};

pub trait Layer {
    fn render(&mut self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder);
    fn resize(&mut self, new_size: PhysicalSize<u32>, queue: &wgpu::Queue);
    fn handle_event(&mut self, event: &WindowEvent, queue: &wgpu::Queue);
}

#[derive(Copy, Clone, Debug)]
pub enum LayerMode {
    Background,
    AlphaBlend,
}

pub struct RenderView {
    size: winit::dpi::PhysicalSize<u32>,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub layers: Vec<Box<dyn Layer>>,
}

impl RenderView {
    pub async fn new(window: &winit::window::Window, audio_file: &str, cli: &Cli) -> Self {
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
        let max_width = limits.max_texture_dimension_2d;
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

        surface.configure(&device, &config);

        //let egui_context = egui::Context::default();
        //let egui_renderer = egui_wgpu::Renderer::new(&device, config.format, 1, 0);

        //let audio_info = audio.info();

        //use plotters::prelude::*;

        //let noise = simdnoise::NoiseBuilder::fbm_1d(256).generate_scaled(0.0, 1.0);

        /*let background_image = noise::NoiseKernelV1 {
            out_width: 1400,
            out_height: 1400,
            scale_x: 10,  // 30, // 10
            scale_y: 280, //150, // 10
            ..noise::NoiseKernelV1::default()
        }
        .make_noise(|tl, _bl, tr, br, d_tl, _d_bl, d_tr, d_br| {
            let r = tl.0[0] as f32 * d_tl;
            let g = br.0[0] as f32 * d_br;
            let b = tr.0[0] as f32 * d_tr;
            let a = br.0[0] as f32;
            Rgba::from([
                r.floor() as u8,
                g.floor() as u8,
                b.floor() as u8,
                a.floor() as u8,
            ])
        });*/

        let mut audio = AudioFile::open(audio_file).await.unwrap();
        let signal = audio.dump_mono();
        let mut analysis = stft(&signal, "hamming", cli.window_size, cli.jump_size);

        analysis.0.truncate(max_width as usize);
        analysis.1.truncate(max_width as usize);

        /*
        dbg!(analysis.0.len());
        dbg!(analysis.0[0].len());
        use ordered_float::OrderedFloat;
        dbg!(analysis
            .0
            .iter()
            .map(|x| { x.iter().map(|x| OrderedFloat(*x)).min() })
            .min());
        dbg!(analysis
            .0
            .iter()
            .map(|x| { x.iter().map(|x| OrderedFloat(*x)).max() })
            .max());
            */
        let _grad = colorgrad::CustomGradient::new()
            .html_colors(&["deeppink", "gold", "seagreen"])
            .build()
            .unwrap();
        let _grad = colorgrad::viridis();
        let grad = colorgrad::CustomGradient::new()
            .colors(&[
                //colorgrad::Color::new(1., 0., 0., 0.),
                //colorgrad::Color::new(0., 1., 0., 0.2),
                //colorgrad::Color::new(0., 0., 1., 1.),
                //colorgrad::Color::new(0., 0., 0., 1.),
                colorgrad::Color::new(0., 0., 0., 0.7),
                colorgrad::Color::new(0., 0., 1., 0.8),
                colorgrad::Color::new(0., 1., 0., 0.9),
                colorgrad::Color::new(1., 0., 0., 1.),
            ])
            .domain(&[-120., -80., -40., 0.])
            .build()
            .unwrap();

        // Map t which is in range [a, b] to range [c, d]
        fn _remap(t: f64, a: f64, b: f64, c: f64, d: f64) -> f64 {
            (t - a) * ((d - c) / (b - a)) + c
        }

        let analysis_image = RgbaImage::from_fn(
            analysis.0.len() as u32,
            (analysis.0[0].len() as f32 * 0.6) as u32,
            |x, y| {
                let val = analysis.0[x as usize][y as usize] as f64;
                //val = remap(val, -140., 0., 0., 1.);
                Rgba(grad.at(val).to_rgba8())
            },
        );
        //let analysis_pass = ShowAnalysisPass::new(
        let analysis_pass = ImageLayerPass::new(
            Some("Analysis Image"),
            image::DynamicImage::ImageRgba8(analysis_image),
            &device,
            &queue,
            &config,
            LayerMode::AlphaBlend,
        );

        //let background_image = load_image("images/noise3.png").await;
        let background_image = load_image("images/baba.png").await;
        let background_pass = ImageLayerPass::new(
            Some("Background Image"),
            background_image,
            &device,
            &queue,
            &config,
            LayerMode::Background,
        );

        let layers = vec![
            Box::new(background_pass) as Box<dyn Layer>,
            Box::new(analysis_pass) as Box<dyn Layer>,
        ];

        RenderView {
            size,
            surface,
            device,
            queue,
            config,
            layers,
        }
    }

    /*
    pub async fn _update_background(&mut self, filename: &str) {
        let background_image = load_image(filename).await;
        let _background = ImageLayerPass::new(
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

    pub fn render(&mut self) {
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.layers.iter_mut().for_each(|layer| {
            layer.render(&view, &mut encoder);
        });
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
