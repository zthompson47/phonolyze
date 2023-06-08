//! crate docs
#![warn(missing_docs)]

mod audio;
mod ease;
mod event;
mod fft;
mod file;
mod layers;
mod render;
mod scale;
mod vertex;

use clap::Parser;
#[allow(unused_imports)]
use image::{Rgba, RgbaImage};
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::{
    audio::AudioFile,
    event::EventHandler,
    fft::stft,
    file::load_image,
    layers::{analysis::AnalysisLayerPass, gui::Gui, scaled_image::ScaledImagePass, LayerMode},
    render::RenderView,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Command line arguments
#[derive(clap::Parser)]
pub struct Cli {
    /// Song file to analyze
    #[arg(default_value = "media/sine.wav")]
    audio_file: String,
    /// Truncate analysis buffer
    #[arg(short, long, default_value_t = 8192)]
    top: usize,
    /// DFT window size
    #[arg(short, long, default_value_t = 2048)]
    window_size: usize,
    /// STFT jump size
    #[arg(short, long, default_value_t = 2048)]
    jump_size: usize,
    /// STFT jump size
    #[arg(short, long, default_value_t = 2048.)]
    latency_ms: f32,
    /// STFT jump size
    #[arg(short, long, default_value_t = 2048)]
    chunk_size: usize,
    /// STFT jump size
    #[arg(short, long, default_value_t = false)]
    play_audio: bool,
}

/// asdf
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn main() {
    // Configure logging
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).unwrap();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _log = tailog::init();
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_maximized(true)
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        window.set_inner_size(winit::dpi::PhysicalSize::new(1280, 960));

        use winit::platform::web::WindowExtWebSys;

        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("phonolyze").unwrap();
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok().unwrap();
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut render_view = RenderView::new(&window).await;
    let cli = Cli::parse();

    #[cfg(not(target_arch = "wasm32"))]
    let _audio_player = crate::audio::AudioPlayer::from(&cli);

    //let background_image = load_image("images/noise3.png").await;
    let background_image = load_image("images/baba.png").await.unwrap();
    let mut audio = AudioFile::open(&cli.audio_file).await.unwrap();
    let signal = audio.dump_mono();
    let analysis = stft(&signal, "hamming", cli.window_size, cli.jump_size);
    let grad = colorgrad::CustomGradient::new()
        .colors(&[
            colorgrad::Color::new(0., 0., 0., 1.),
            colorgrad::Color::new(0., 0., 1., 1.),
            colorgrad::Color::new(0., 1., 0., 1.),
            colorgrad::Color::new(1., 0., 0., 1.),
        ])
        .domain(&[-150., -80., -40., 0.])
        .build()
        .unwrap();

    render_view.capture_layer(move |device, queue, config| {
        let background_image = ScaledImagePass::new(
            Some("Background Image"),
            background_image,
            device,
            queue,
            config,
            LayerMode::Background,
        );

        Box::new(background_image)
    });

    /*
    let analysis_image = RgbaImage::from_fn(
        analysis.0.len() as u32,
        (analysis.0[0].len() as f32 * 0.6) as u32,
        |x, y| {
            let val = analysis.0[x as usize][y as usize] as f64;
            //val = remap(val, -140., 0., 0., 1.);
            Rgba(grad.at(val).to_rgba8())
        },
    );

    render_view.capture_layer(move |device, queue, config| {
        let analysis_pass_scaled = ScaledImagePass::new(
            Some("Analysis Image Scaled"),
            image::DynamicImage::ImageRgba8(analysis_image),
            device,
            queue,
            config,
            LayerMode::AlphaBlend,
        );

        Box::new(analysis_pass_scaled)
    });
    */

    render_view.capture_layer(move |device, queue, config| {
        Box::new(AnalysisLayerPass::new(
            Some("Analysis Pass"),
            analysis.0,
            device,
            queue,
            config,
            LayerMode::AlphaBlend,
            grad,
        ))
    });

    render_view.capture_layer(|device, _queue, config| {
        Box::new(Gui::new(device, &event_loop, config.format))
    });

    let mut event_handler = EventHandler::new(window, render_view);

    event_loop.run(move |event, _, control_flow| {
        event_handler.handle_event(event, control_flow);
    });
}

//dbg!(analysis.0.len());
//dbg!(analysis.0[0].len());
/*
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
