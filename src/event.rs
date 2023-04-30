use winit::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    window::Window,
};

use crate::render::RenderView;

pub struct EventHandler {
    pub window: Window,
    pub render_view: RenderView,
}

impl EventHandler {
    pub fn handle_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == self.window.id() => *control_flow = ControlFlow::Exit,

            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                window_id,
            } if window_id == self.window.id() => {
                if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                    *control_flow = ControlFlow::Exit
                }
            }

            Event::MainEventsCleared => {
                let now = instant::Instant::now();
                let dt = now - self.render_view.last_updated;

                self.render_view.last_updated = now;
                self.render_view.update(dt);

                self.window.request_redraw();
            }

            Event::RedrawRequested(window_id) if window_id == self.window.id() => {
                let frame = self.render_view.surface.get_current_texture().unwrap();

                self.render_view.render(frame);
            }

            _ => (),
        }
    }
}
