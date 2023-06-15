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
    fn render(&mut self, _renderer: &mut Renderer) {}
    fn resize(&mut self, _new_size: PhysicalSize<u32>, _queue: &wgpu::Queue) {}
    fn update(
        &mut self,
        _delta: instant::Duration,
        _state: &mut LayerState,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LayerMode {
    Background,
    AlphaBlend,
}

#[derive(Default)]
pub struct LayerState {
    pub color_map: ColorMap,
    pub prev_color_map: Option<ColorMap>,
}

impl LayerState {
    fn update_color_map(&mut self) -> Option<ColorMap> {
        let mut result = None;

        if let Some(prev_color_map) = self.prev_color_map {
            if prev_color_map != self.color_map {
                self.prev_color_map = Some(self.color_map);
                result = self.prev_color_map;
            }
        } else {
            self.prev_color_map = Some(self.color_map);
            result = self.prev_color_map;
        }

        result
    }
}
