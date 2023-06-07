pub mod analysis;
pub mod gui;
pub mod scaled_image;

use winit::{dpi::PhysicalSize, event::WindowEvent};

use crate::{layers::gui::ColorMap, render::Renderer};

pub trait Layer {
    fn handle_event(
        &mut self,
        _event: &WindowEvent,
        _queue: &wgpu::Queue,
    ) -> egui_winit::EventResponse {
        egui_winit::EventResponse {
            consumed: false,
            repaint: false,
        }
    }
    fn render(&mut self, renderer: &mut Renderer);
    fn resize(&mut self, _new_size: PhysicalSize<u32>, _queue: &wgpu::Queue) {}
    //fn update(&mut self, _delta: instant::Duration, _state: &mut LayerState) {}
}

#[derive(Copy, Clone, Debug)]
pub enum LayerMode {
    Background,
    AlphaBlend,
}

#[derive(Default)]
pub struct LayerState {
    pub color_map: ColorMap,
}
