//#![deny(elided_lifetimes_in_paths)]
mod audio;
mod color;
mod ease;
mod event;
mod fft;
mod layers;
mod render;
mod resource;
mod uniforms;

use clap::Parser;
use layers::meter::MeterPass;
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::{
    audio::AudioFile,
    event::EventHandler,
    fft::stft,
    layers::{analysis::AnalysisLayerPass, gui::Gui, scaled_image::ScaledImagePass, LayerMode},
    render::RenderView,
    resource::load_image,
    uniforms::{ColorMap, Gradient},
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
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_maximized(true)
        //.with_inner_size(winit::dpi::PhysicalSize::new(1280, 960))
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    //if cfg!(target_arch = "wasm32")
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.

        use winit::platform::web::WindowExtWebSys;
        //window.set_inner_size(winit::dpi::PhysicalSize::new(1280, 960));

        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("phonolyze").unwrap();

                let canvas = window.canvas().unwrap();
                //let mut surface = Surface::from_canvas(canvas.clone()).unwrap();

                let canvas = web_sys::Element::from(window.canvas().unwrap());
                dst.append_child(&canvas).ok().unwrap();
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    #[cfg(not(target_arch = "wasm32"))]
    let audio_player = crate::audio::AudioPlayer::from(&cli);

    let mut ctx = RenderView::new(&window).await;

    #[cfg(not(target_arch = "wasm32"))]
    {
        ctx.state.progress = Some(audio_player.progress.clone());
    }

    //let background_image = load_image("images/noise3.png").await.unwrap();
    let background_image = load_image("images/baba.png").await.unwrap();

    let background_image_pass = Box::new(ScaledImagePass::new(
        background_image,
        &ctx.device,
        &ctx.queue,
        &ctx.config,
        LayerMode::Background,
    ));

    let mut audio = AudioFile::open(&cli.audio_file).await.unwrap();
    let signal = audio.dump_mono(cli.seconds);
    dbg!(&signal.len());
    let analysis = stft(&signal, "hamming", cli.window_size, cli.jump_size);
    dbg!(cli.window_size, cli.jump_size);
    dbg!(&analysis.0.len(), &analysis.0[0].len());

    #[cfg(not(target_arch = "wasm32"))]
    if let Ok(mut progress) = audio_player.progress.lock() {
        progress.music_length = signal.len() as f64 / audio.sample_rate() as f64;
    }

    dbg!(&analysis.0.len(), &analysis.0[0].len());

    let analysis_pass = Box::new(AnalysisLayerPass::new(
        &analysis.0,
        &ctx,
        LayerMode::AlphaBlend,
        Gradient::new(
            Some("InitGradient"),
            ColorMap::default().uniform(),
            &ctx.device,
            &ctx.queue,
        ),
    ));

    let meter_pass = Box::new(MeterPass::new(&analysis.0, &ctx));

    let gui_pass = Box::new(Gui::new(
        &event_loop,
        &ctx.device,
        ctx.config.format,
        ctx.scale_factor,
    ));

    ctx.layers.push(background_image_pass);
    ctx.layers.push(analysis_pass);
    ctx.layers.push(meter_pass);
    ctx.layers.push(gui_pass);

    let mut event_handler = EventHandler::new(&window, ctx);

    let _ = event_loop.run(move |event, elwt| {
        event_handler.handle_event(event, elwt);
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
