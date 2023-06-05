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
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::layers::gui::Gui;

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

    let cli = Cli::parse();

    #[cfg(not(target_arch = "wasm32"))]
    let _audio_player = crate::audio::AudioPlayer::from(&cli);

    let mut render_view = render::RenderView::new(&window, &cli.audio_file, &cli).await;
    let gui = Gui::new(&render_view.device, &event_loop, render_view.config.format);

    render_view.push_layer(Box::new(gui));

    let mut event_handler = event::EventHandler::new(window, render_view);

    event_loop.run(move |event, _, control_flow| {
        event_handler.handle_event(event, control_flow);
    });
}
