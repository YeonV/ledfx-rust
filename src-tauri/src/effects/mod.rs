// src-tauri/src/effects/mod.rs

use crate::audio::AudioAnalysisData;

pub trait Effect: Send {
    fn render_frame(&mut self, audio_data: &AudioAnalysisData) -> Vec<u8>;
}

pub mod blade;
pub mod legacy;