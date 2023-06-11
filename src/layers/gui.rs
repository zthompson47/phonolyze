use std::fmt::Display;

use winit::event::WindowEvent;

use crate::render::Renderer;

use super::Layer;

pub struct Gui {
    context: egui::Context,
    renderer: egui_wgpu::Renderer,
    window_state: egui_winit::State,
}

impl Gui {
    pub fn new(
        device: &wgpu::Device,
        event_loop: &winit::event_loop::EventLoop<()>,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        Gui {
            context: egui::Context::default(),
            renderer: egui_wgpu::Renderer::new(device, texture_format, None, 1),
            window_state: egui_winit::State::new(event_loop),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum ColorMap {
    Rgb,
    #[default]
    Blue,
}

impl ColorMap {
    #[allow(dead_code)]
    pub fn grad(&self) -> colorgrad::Gradient {
        match &self {
            Self::Rgb => colorgrad::CustomGradient::new()
                .colors(&[
                    colorgrad::Color::new(0.0, 0.0, 0.0, 1.0),
                    colorgrad::Color::new(0.0, 0.0, 1.0, 1.0),
                    colorgrad::Color::new(0.0, 1.0, 0.0, 1.0),
                    colorgrad::Color::new(1.0, 0.0, 0.0, 1.0),
                ])
                .domain(&[-150., -80., -40., 0.])
                .build()
                .unwrap(),
            Self::Blue => colorgrad::CustomGradient::new()
                .colors(&[
                    colorgrad::Color::new(0.0, 0.0, 0.0, 0.0),
                    colorgrad::Color::new(0.0, 0.0, 1.0, 0.5),
                    colorgrad::Color::new(0.0, 0.2, 0.5, 1.0),
                    colorgrad::Color::new(0.2, 0.2, 1.0, 1.0),
                ])
                .domain(&[-150., -80., -40., 0.])
                .build()
                .unwrap(),
        }
    }
}

impl Display for ColorMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Rgb => write!(f, "Rgb"),
            Self::Blue => write!(f, "Blue"),
        }
    }
}

impl Layer for Gui {
    fn handle_event(
        &mut self,
        event: &WindowEvent,
        _queue: &wgpu::Queue,
    ) -> egui_winit::EventResponse {
        self.window_state.on_event(&self.context, event)
    }

    fn render(&mut self, renderer: &mut Renderer) {
        let input = self.window_state.take_egui_input(renderer.window);
        let output = {
            use ColorMap::*;
            self.context.run(input, |ctx| {
                egui::Area::new("testitout").show(ctx, |ui| {
                    egui::ComboBox::from_label("Colormap")
                        .selected_text(format!("{:?}", renderer.state.color_map))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut renderer.state.color_map,
                                Rgb,
                                Rgb.to_string(),
                            );
                            ui.selectable_value(
                                &mut renderer.state.color_map,
                                Blue,
                                Blue.to_string(),
                            );
                        });
                });
            })
        };

        // Keep redrawing for animations.  TODO: set a timer for non-zero durations
        if output.repaint_after.is_zero() {
            renderer.window.request_redraw();
        }

        let clipped_primitives: Vec<egui::epaint::ClippedPrimitive> =
            self.context.tessellate(output.shapes);

        for (id, image_delta) in &output.textures_delta.set {
            self.renderer
                .update_texture(renderer.device, renderer.queue, *id, image_delta);
        }

        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [renderer.config.width, renderer.config.height],
            pixels_per_point: 2.0, //self.scale_factor,
        };

        self.renderer.update_buffers(
            renderer.device,
            renderer.queue,
            renderer.encoder,
            clipped_primitives.as_slice(),
            &screen_descriptor,
        );

        {
            let mut render_pass = renderer
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: renderer.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

            self.renderer.render(
                &mut render_pass,
                clipped_primitives.as_slice(),
                &screen_descriptor,
            );
        }
    }

    fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>, _queue: &wgpu::Queue) {}
}
