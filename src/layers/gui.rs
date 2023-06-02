use winit::{event::WindowEvent, window::Window};

use crate::render::{Layer, LayerState};

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

#[derive(Debug, Default, PartialEq)]
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

impl std::fmt::Display for ColorMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Rgb => write!(f, "Rgb"),
            Self::Blue => write!(f, "Blue"),
        }
    }
}

/*
struct EventResult {
    repaint: bool,
    consumed: bool,
}
*/

impl Layer for Gui {
    fn handle_event(
        &mut self,
        event: &WindowEvent,
        _queue: &wgpu::Queue,
    ) -> egui_winit::EventResponse {
        self.window_state.on_event(&self.context, event)
        /*let response = self.window_state.on_event(&self.context, event);

        self.repaint = response.repaint;

        response.consumed*/
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        state: &mut LayerState,
        //renderer: &mut RenderView,
    ) {
        let input = self.window_state.take_egui_input(window);
        let output = {
            use ColorMap::*;
            self.context.run(input, |ctx| {
                egui::Area::new("testitout").show(ctx, |ui| {
                    egui::ComboBox::from_label("Colormap")
                        .selected_text(format!("{:?}", state.color_map))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.color_map, Rgb, Rgb.to_string());
                            ui.selectable_value(&mut state.color_map, Blue, Blue.to_string());
                        });
                });
            })
        };

        let clipped_primitives: Vec<egui::epaint::ClippedPrimitive> =
            self.context.tessellate(output.shapes);

        for (id, image_delta) in &output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }

        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [config.width, config.height],
            pixels_per_point: 2.0, //self.scale_factor,
        };

        self.renderer.update_buffers(
            device,
            queue,
            encoder,
            clipped_primitives.as_slice(),
            &screen_descriptor,
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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
}
