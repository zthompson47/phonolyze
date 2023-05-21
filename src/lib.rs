mod audio;
mod event;
mod fft;
mod file;
mod render;
mod texture;
mod vertex;

use clap::Parser;
use winit::{event_loop::EventLoop, window::WindowBuilder};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(clap::Parser)]
pub struct Cli {
    audio_file: Option<String>,
    #[arg(short, long, default_value_t = 0.6)]
    top: f32,
    #[arg(short, long, default_value_t = 2048)]
    window_size: usize,
    #[arg(short, long, default_value_t = 2048)]
    jump_size: usize,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn main() {
    // Configure logging
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).unwrap();
        } else {
            env_logger::init();
        }
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
                let dst = doc.get_element_by_id("soundy")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // Command line args
    let cli = Cli::parse();
    let audio_file = cli.audio_file.clone().unwrap_or(String::from("media/_song.flac"));
    let render_view = render::RenderView::new(&window, &audio_file, &cli).await;
    let mut event_handler = event::EventHandler::new(window, render_view);

    event_loop.run(move |event, _, control_flow| {
        event_handler.handle_event(event, control_flow);
    });
}
