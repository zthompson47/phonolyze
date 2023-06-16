mod audio;
mod camera;
mod ease;
mod event;
mod fft;
mod file;
mod gradient;
mod layers;
mod render;
mod scale;
mod vertex;

use clap::Parser;
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::{
    audio::AudioFile,
    event::EventHandler,
    fft::stft,
    file::load_image,
    gradient::Gradient,
    layers::{
        analysis::AnalysisLayerPass,
        gui::{ColorMap, Gui},
        scaled_image::ScaledImagePass,
        LayerMode,
    },
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
    /// Seconds to analyze
    #[arg(short, long)]
    seconds: Option<f32>,
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

/// Launch winit or wasm.
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

    let cli = Cli::parse();
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

    let background_image = load_image("images/noise3.png").await.unwrap();
    //let background_image = load_image("images/baba.png").await.unwrap();

    render_view.capture_layer(move |device, queue, config, _scale_factor| {
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

    let mut audio = AudioFile::open(&cli.audio_file).await.unwrap();
    let signal = audio.dump_mono(cli.seconds);
    let analysis = stft(&signal, "hamming", cli.window_size, cli.jump_size);

    #[cfg(not(target_arch = "wasm32"))]
    let audio_player = crate::audio::AudioPlayer::from(&cli);

    render_view.capture_layer(move |device, queue, config, _scale_factor| {
        Box::new(AnalysisLayerPass::new(
            Some("Analysis Pass"),
            analysis.0,
            device,
            config,
            LayerMode::AlphaBlend,
            Gradient::new(Some("InitGradient"), ColorMap::default().grad(), device),
            audio_player.progress.clone(),
            signal.len() as u32,
        ))
    });

    render_view.capture_layer(|device, _queue, config, scale_factor| {
        Box::new(Gui::new(device, &event_loop, config.format, scale_factor))
    });

    let mut event_handler = EventHandler::new(window, render_view);

    event_loop.run(move |event, _, control_flow| {
        event_handler.handle_event(event, control_flow);
    });
}

    //let grad = ColorMap::Rgb.grad();
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

    render_view.capture_layer(move |device, queue, config, _scale_factor| {
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
