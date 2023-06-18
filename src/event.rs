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
        if let Event::WindowEvent {
            ref event,
            window_id,
        } = event
        {
            if window_id == self.window.id() {
                let mut consumed = false;
                let mut repaint = false;

                self.render_view.layers.iter_mut().for_each(|layer| {
                    let response = layer.handle_event(event, &self.render_view.queue);

                    if response.consumed {
                        consumed = true;
                    }
                    if response.repaint {
                        repaint = true;
                    }
                });

                if repaint {
                    self.window.request_redraw();
                }
                if consumed {
                    return;
                }
            }
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == self.window.id() => *control_flow = ControlFlow::Exit,

            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                window_id,
            } if window_id == self.window.id() => {
                if let Some(VirtualKeyCode::Escape | VirtualKeyCode::Q) = input.virtual_keycode {
                    *control_flow = ControlFlow::Exit
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
                self.render_view.update(delta, &self.window);
                //self.window.request_redraw();
            }

            Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                self.render_view.render(&self.window);
            }

            _ => (),
        }
    }
}
