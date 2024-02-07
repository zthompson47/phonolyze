use strum::IntoEnumIterator;
use winit::event::WindowEvent;

use crate::{render::Renderer, uniforms::ColorMap};

use super::{Layer, LayerState};

pub struct Gui {
    context: egui::Context,
    renderer: egui_wgpu::Renderer,
    window_state: egui_winit::State,
    pixels_per_point: f32,
}

impl Gui {
    pub fn new(
        event_loop: &winit::event_loop::EventLoop<()>,
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
        scale_factor: f32,
    ) -> Self {
        let context = egui::Context::default();
        let viewport_id = context.viewport_id();
        let display_target = event_loop;
        let native_pixels_per_point = Some(scale_factor);
        let max_texture_side = None;
        Gui {
            context: egui::Context::default(),
            renderer: egui_wgpu::Renderer::new(device, texture_format, None, 1),
            window_state: egui_winit::State::new(
                context,
                viewport_id,
                display_target,
                native_pixels_per_point,
                max_texture_side,
            ),
            pixels_per_point: scale_factor,
        }
    }
}

impl Layer for Gui {
    fn handle_event(
        &mut self,
        event: &WindowEvent,
        _queue: &wgpu::Queue,
        _state: &mut LayerState,
        window: &winit::window::Window,
    ) -> egui_winit::EventResponse {
        self.window_state.on_window_event(window, event)
    }

    fn render(&mut self, renderer: &mut Renderer, state: &mut LayerState) {
        let input = self.window_state.take_egui_input(renderer.window);
        let output = {
            println!(
                "-000--------------------{:?}---{:?}--------------------",
                self.context.pixels_per_point(),
                self.pixels_per_point
            );
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
        //if output.repaint_after.is_zero() {
        renderer.window.request_redraw();
        //}
        println!(
            "-000.555--------------------{:?}---{:?}--------------------",
            self.context.pixels_per_point(),
            self.pixels_per_point
        );

        let clipped_primitives: Vec<egui::epaint::ClippedPrimitive> = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);
        println!(
            "-111-------------{:?}--{:?}----------------------------",
            self.context.pixels_per_point(),
            self.pixels_per_point
        );

        for (id, image_delta) in &output.textures_delta.set {
            self.renderer
                .update_texture(renderer.device, renderer.queue, *id, image_delta);
        }

        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
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
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: renderer.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
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

    fn resize(
        &mut self,
        _new_size: winit::dpi::PhysicalSize<u32>,
        _queue: &wgpu::Queue,
        _state: &mut LayerState,
    ) {
    }
}
