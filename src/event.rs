use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoopWindowTarget,
    keyboard::{Key, NamedKey},
    window::Window,
};

use crate::render::RenderView;

pub struct EventHandler<'a> {
    pub window: &'a Window,
    pub render_view: RenderView<'a>,
    last_updated: instant::Instant,
}

impl<'a> EventHandler<'a> {
    pub fn new(window: &'a Window, render_view: RenderView<'a>) -> Self {
        EventHandler {
            window,
            render_view,
            last_updated: instant::Instant::now(),
        }
    }

    pub fn handle_event(&mut self, event: Event<()>, elwt: &EventLoopWindowTarget<()>) {
        if let Event::WindowEvent {
            ref event,
            window_id,
        } = event
        {
            if window_id == self.window.id() {
                let mut consumed = false;
                let mut repaint = false;

                self.render_view.layers.iter_mut().for_each(|layer| {
                    let response = layer.handle_event(
                        event,
                        &self.render_view.queue,
                        &mut self.render_view.state,
                        self.window,
                    );

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
                event: WindowEvent::ModifiersChanged(mods),
                window_id,
            } if window_id == self.window.id() => {
                dbg!(mods);
                self.render_view.state.modifiers = mods.state();
            }

            Event::WindowEvent {
                event:
                    WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        ..//inner_size_writer, // TODO: what is inner_size_writer?
                    },
                window_id,
            } if window_id == self.window.id() => {
                self.render_view.scale_factor = scale_factor as f32
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == self.window.id() => elwt.exit(),

            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                window_id,
            } if window_id == self.window.id() => {
                if let Key::Named(NamedKey::Escape) | Key::Character("Q") =
                    event.logical_key.as_ref()
                {
                    elwt.exit()
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(physical_size),
                window_id,
            } if window_id == self.window.id() => {
                dbg!("resize", physical_size);
                self.render_view.resize(physical_size);
                self.window.request_redraw();
            }

            Event::AboutToWait => {
                let now = instant::Instant::now();
                let delta = now - self.last_updated;

                self.last_updated = now;
                self.render_view.update(delta, self.window);
                //self.window.request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == self.window.id() => {
                self.render_view.render(self.window);
            }

            _ => (),
        }
    }
}
