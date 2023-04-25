mod view;

use winit::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).unwrap();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        tracing_subscriber::fmt().init();
    }

    tracing::info!("Running...");

    // Window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let render = view::RenderState::new(&window).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == window.id() => *control_flow = ControlFlow::Exit,

        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            window_id,
        } if window_id == window.id() => {
            if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                *control_flow = ControlFlow::Exit
            }
        }

        Event::RedrawRequested(window_id) if window_id == window.id() => {
            tracing::info!("Redraw requested");
            let output = render.surface.get_current_texture().unwrap();
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = render
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            {
                let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.3,
                                b: 0.1,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });
            }
            render.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }

        _ => (),
    })
}
