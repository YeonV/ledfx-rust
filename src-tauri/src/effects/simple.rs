// src-tauri/src/effects/simple.rs

use crate::audio::AudioAnalysisData;
use crate::effects::Effect;
use crate::utils::hsv_to_rgb;

// --- The Rainbow Effect ---
pub struct RainbowEffect {
    pub hue: f32,
}
impl Effect for RainbowEffect {
    fn render_frame(&mut self, pixel_count: u32, _audio_data: &AudioAnalysisData) -> Vec<u8> {
        self.hue = (self.hue + 1.0) % 360.0;
        let rgb = hsv_to_rgb(self.hue, 1.0, 1.0);
        let mut data = Vec::with_capacity((pixel_count * 3) as usize);
        for _ in 0..pixel_count {
            data.extend_from_slice(&rgb);
        }
        data
    }
    fn update_settings(&mut self, _settings: serde_json::Value) {}
}

// --- The Scan Effect ---
pub struct ScanEffect {
    pub position: u32,
    pub color: [u8; 3],
}
impl Effect for ScanEffect {
    fn render_frame(&mut self, pixel_count: u32, _audio_data: &AudioAnalysisData) -> Vec<u8> {
        self.position = (self.position + 1) % pixel_count;
        let mut data = vec![0; (pixel_count * 3) as usize];
        let start_index = (self.position * 3) as usize;
        if start_index + 3 <= data.len() {
            data[start_index..start_index + 3].copy_from_slice(&self.color);
        }
        data
    }
    fn update_settings(&mut self, _settings: serde_json::Value) {}
}

// --- The Scroll Effect ---
pub struct ScrollEffect {
    pub hue: f32,
}
impl Effect for ScrollEffect {
    fn render_frame(&mut self, pixel_count: u32, _audio_data: &AudioAnalysisData) -> Vec<u8> {
        self.hue = (self.hue + 0.5) % 360.0;
        let mut data = Vec::with_capacity((pixel_count * 3) as usize);
        for i in 0..pixel_count {
            let pixel_hue = (self.hue + (i as f32 * (360.0 / pixel_count as f32))) % 360.0;
            let rgb = hsv_to_rgb(pixel_hue, 1.0, 1.0);
            data.extend_from_slice(&rgb);
        }
        data
    }
    fn update_settings(&mut self, _settings: serde_json::Value) {}
}
