// src-tauri/src/effects/mod.rs

use crate::audio::AudioAnalysisData;
use as_any::AsAny;

pub trait Effect: Send + AsAny {
    fn render_frame(&mut self, audio_data: &AudioAnalysisData) -> Vec<u8>;
    // --- NEW: The method for live updates ---
    fn update_settings(&mut self, settings: serde_json::Value);
}

pub mod blade;
pub mod legacy;