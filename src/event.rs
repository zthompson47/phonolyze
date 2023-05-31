use winit::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

use crate::render::RenderView;

pub struct EventHandler {
    pub window: Window,
    pub render_view: RenderView,
    last_updated: instant::Instant,
}

impl EventHandler {
    pub fn new(window: Window, render_view: RenderView) -> Self {
        EventHandler {
            window,
            render_view,
            last_updated: instant::Instant::now(),
        }
    }

    pub fn handle_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == self.window.id() => *control_flow = ControlFlow::Exit,

            Event::WindowEvent {
                event: event @ WindowEvent::KeyboardInput { .. },
                window_id,
            } if window_id == self.window.id() => {
                self.render_view.layers.iter_mut().for_each(|layer| {
                    if layer.handle_event(&event, &self.render_view.queue) {
                        #[allow(clippy::needless_return)] // TODO - handle consumed events
                        return;
                    }
                });

                self.window.request_redraw();

                if let WindowEvent::KeyboardInput { input, .. } = event {
                    if let Some(VirtualKeyCode::Escape | VirtualKeyCode::Q) = input.virtual_keycode {
                        *control_flow = ControlFlow::Exit
                  }
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                window_id,
            } if window_id == self.window.id() => {
                self.render_view.resize(physical_size);
                self.window.request_redraw();
            }

            Event::MainEventsCleared => {
                let now = instant::Instant::now();
                let delta = now - self.last_updated;

                self.last_updated = now;
                self.render_view.update(delta);
                //self.window.request_redraw();
            }

            Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                self.render_view.render(&self.window);
            }

            _ => (),
        }
    }
}
