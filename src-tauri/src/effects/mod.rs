// src-tauri/src/effects/mod.rs

use crate::audio::AudioAnalysisData;
use as_any::AsAny;

// --- THE FIX: Add pixel_count back to the render_frame method ---
pub trait Effect: Send + AsAny {
    fn render_frame(&mut self, pixel_count: u32, audio_data: &AudioAnalysisData) -> Vec<u8>;
    fn update_settings(&mut self, settings: serde_json::Value);
}

pub mod blade;
pub mod legacy;
pub mod simple;
