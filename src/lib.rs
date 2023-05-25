mod audio;
mod event;
mod fft;
mod file;
mod render;
mod scale;
mod texture;
mod vertex;

use anyhow::Error;
use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait};
use winit::{event_loop::EventLoop, window::WindowBuilder};

use crate::audio::AudioPlayer;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(clap::Parser)]
pub struct Cli {
    audio_file: Option<String>,
    #[arg(short, long, default_value_t = 8192)]
    top: usize,
    #[arg(short, long, default_value_t = 2048)]
    window_size: usize,
    #[arg(short, long, default_value_t = 2048)]
    jump_size: usize,
    #[arg(short, long, default_value_t = 2048.)]
    latency_ms: f32,
    #[arg(short, long, default_value_t = 2048)]
    chunk_size: usize,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn main() {
    // Configure logging
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).unwrap();
        } else {
            let _log = tailog::init();
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
                let dst = doc.get_element_by_id("phonolyze").unwrap();
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok().unwrap();
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let cli = Cli::parse();
    let audio_file = cli
        .audio_file
        .clone()
        .unwrap_or(String::from("media/jtree_stream.m4a"));

    #[cfg(not(target_arch = "wasm32"))]
    {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or(Error::msg("No audio device found")).unwrap();
        let config = device.default_output_config().unwrap();
        let audio_player = match config.sample_format() {
            cpal::SampleFormat::I8 => {
                AudioPlayer::new::<i8>(&device, &config.into(), cli.latency_ms, cli.chunk_size).await
            }
            cpal::SampleFormat::F32 => {
                AudioPlayer::new::<f32>(&device, &config.into(), cli.latency_ms, cli.chunk_size).await
            }
            _ => panic!("unsupported format"),
        }
        .unwrap();

        audio_player.play(audio_file.clone().into());
    }

    let render_view = render::RenderView::new(&window, &audio_file, &cli).await;
    let mut event_handler = event::EventHandler::new(window, render_view);

    event_loop.run(move |event, _, control_flow| {
        event_handler.handle_event(event, control_flow);
    });
}
