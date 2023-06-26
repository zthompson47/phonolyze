pub mod analysis;
pub mod gui;
pub mod meter;
pub mod scaled_image;

use std::sync::{Arc, Mutex};

use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::{audio::PlaybackPosition, layers::gui::ColorMap, render::Renderer, uniforms::Scale};

#[allow(unused_variables)]
pub trait Layer {
    fn handle_event(
        &mut self,
        event: &WindowEvent,
        queue: &wgpu::Queue,
        //state: &mut LayerState,
    ) -> egui_winit::EventResponse {
        egui_winit::EventResponse {
            consumed: false,
            repaint: false,
        }
    }
    fn render(&mut self, renderer: &mut Renderer, state: &mut LayerState) {}
    fn resize(&mut self, new_size: PhysicalSize<u32>, queue: &wgpu::Queue, state: &mut LayerState) {
    }
    fn update(
        &mut self,
        delta: instant::Duration,
        state: &mut LayerState,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: &Window,
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
    pub progress: Option<Arc<Mutex<PlaybackPosition>>>,
    pub scale: Option<Scale>,
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
