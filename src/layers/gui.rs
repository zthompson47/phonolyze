use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};
use winit::event::WindowEvent;

use crate::{uniforms::InnerGradient, render::Renderer};

use super::{Layer, LayerState};

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
        scale_factor: f32,
    ) -> Self {
        Gui {
            context: egui::Context::default(),
            renderer: egui_wgpu::Renderer::new(device, texture_format, None, 1),
            window_state: {
                let mut window_state = egui_winit::State::new(event_loop);
                window_state.set_pixels_per_point(scale_factor);
                window_state
            },
        }
    }
}

#[derive(Copy, Clone, Debug, Default, EnumIter, Display, PartialEq)]
pub enum ColorMap {
    Blue,
    #[default]
    Rgb,
    RgbInv,
    Crazy,
}

impl ColorMap {
    #[allow(dead_code)]
    pub fn grad(&self) -> InnerGradient {
        match &self {
            Self::Rgb => InnerGradient {
                r: [0.0, 0.0, 0.0, 1.0],
                g: [0.0, 0.0, 1.0, 0.0],
                b: [0.0, 1.0, 0.0, 0.0],
                a: [0.0, 0.8, 1.0, 1.0],
                domain: [-150.0, -80.0, -40.0, 0.0],
            },
            Self::Blue => InnerGradient {
                r: [0.0, 0.0, 0.0, 0.2],
                g: [0.0, 0.0, 0.2, 0.2],
                b: [0.0, 1.0, 0.5, 1.0],
                a: [0.0, 0.8, 1.0, 1.0],
                domain: [-150.0, -80.0, -40.0, 0.0],
            },
            Self::RgbInv => InnerGradient {
                r: [1.0, 0.0, 0.0, 0.0],
                g: [0.0, 1.0, 0.0, 0.0],
                b: [0.0, 0.0, 1.0, 0.0],
                a: [1.0, 1.0, 0.8, 0.0],
                domain: [-150.0, -80.0, -40.0, 0.0],
            },
            Self::Crazy => InnerGradient {
                r: [1.0, 0.2, 0.8, 0.2],
                g: [0.0, 1.0, 0.0, 0.5],
                b: [0.2, 0.0, 0.7, 0.3],
                a: [1.0, 1.0, 0.8, 0.0],
                domain: [-150.0, -100.0, -80.0, 0.0],
            },
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

    fn render(&mut self, renderer: &mut Renderer, state: &mut LayerState) {
        let input = self.window_state.take_egui_input(renderer.window);
        let output = {
            self.context.run(input, |ctx| {
                egui::Area::new("testitout").show(ctx, |ui| {
                    egui::ComboBox::from_label("Colormap")
                        .selected_text(format!("{:?}", state.color_map))
                        .show_ui(ui, |ui| {
                            for color in ColorMap::iter() {
                                ui.selectable_value(&mut state.color_map, color, color.to_string());
                            }
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
            pixels_per_point: renderer.scale_factor,
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
