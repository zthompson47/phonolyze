#![allow(unused)]
use image::Rgba;
use winit::dpi::PhysicalSize;

use crate::{audio::AudioFile, fft::stft, file, texture::TiledBackgroundPass};

#[allow(unused)]
pub struct RenderView {
    size: winit::dpi::PhysicalSize<u32>,
    scale_factor: f32,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    background: TiledBackgroundPass,
}

impl RenderView {
    pub async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
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

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    //features: wgpu::Features::empty(),
                    features: wgpu::Features::PUSH_CONSTANTS,

                    #[cfg(target_arch = "wasm32")]
                    limits: wgpu::Limits::downlevel_webgl2_defaults(),

                    #[cfg(not(target_arch = "wasm32"))]
                    limits: wgpu::Limits::default(),
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

        //let background_image = file::load_image("images/noise3.png").await;

        /*let background_image = file::load_image("images/baba.png").await;
        let background = TiledBackgroundPass::new(
            Some("Background Image"),
            background_image,
            &device,
            &queue,
            &config,
        );*/

        //let egui_context = egui::Context::default();
        //let egui_renderer = egui_wgpu::Renderer::new(&device, config.format, 1, 0);

        //let noise = simdnoise::NoiseBuilder::fbm_1d(256).generate_scaled(0.0, 1.0);

        //let mut audio = AudioFile::open("/home/zach/ph2022-12-28S1t.09.flac").unwrap();
        let mut audio = AudioFile::open("media/_song.flac").await.unwrap();
        //let mut audio = AudioFile::open("media/jtree_stream.m4a").await.unwrap();
        /*let mut audio = AudioFile::open(
            "/home/zach/tunes/The Losing End (When You're On) (2009 Remaster)-3Bd-dDZMoX4.flac",
        )
        .unwrap();*/

        let signal = audio.dump_mono();
        dbg!(signal.len());
        //let analysis = analyze(&signal[0..1024 * 4000], "hamming", 1024, 1024);
        //let analysis = stft(&signal[0..1024 * 4000], "hamming", 2048, 2048);
        let analysis = stft(&signal, "hamming", 2048, 2048);
        dbg!(analysis.0.len());

        use ordered_float::OrderedFloat;
        dbg!(analysis.0.iter().map(|x| OrderedFloat(x[0])).min());
        dbg!(analysis.0.iter().map(|x| OrderedFloat(x[0])).max());

        //let audio_info = audio.info();

        //use plotters::prelude::*;

        dbg!(analysis.0.len() as u32);
        dbg!(analysis.0[0].len() as u32);

        let background_image = image::RgbaImage::from_fn(
            analysis.0.len() as u32,
            analysis.0[0].len() as u32,
            |x, y| {
                let val = analysis.0[x as usize][y as usize];
                //dbg!(val);
                image::Rgba::from([(val * 60.) as u8, (val * 60.) as u8, (val * 60.) as u8, 255])
            },
        );

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

        let background = TiledBackgroundPass::new(
            Some("Background Image"),
            image::DynamicImage::ImageRgba8(background_image),
            &device,
            &queue,
            &config,
        );

        RenderView {
            size,
            scale_factor,
            surface,
            device,
            queue,
            config,
            background,
        }
    }

    pub async fn _update_background(&mut self, filename: &str) {
        let background_image = file::load_image(filename).await;
        let _background = TiledBackgroundPass::new(
            Some("Background Image"),
            background_image,
            &self.device,
            &self.queue,
            &self.config,
        );
    }

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
            self.background.resize(new_size, &self.queue);
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

        self.background.render(&view, &mut encoder);

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
