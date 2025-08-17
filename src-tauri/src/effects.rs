// src-tauri/src/effects.rs

use crate::audio::AudioAnalysisData;
use crate::utils::hsv_to_rgb; // Import from our new utils module

// The Effect Trait now receives audio data.
pub trait Effect: Send {
    fn render_frame(&mut self, leds: u32, audio_data: &AudioAnalysisData) -> Vec<u8>;
}

// --- The Rainbow Effect ---
pub struct RainbowEffect { pub hue: f32 }
impl Effect for RainbowEffect {
    fn render_frame(&mut self, leds: u32, _audio_data: &AudioAnalysisData) -> Vec<u8> {
        self.hue = (self.hue + 1.0) % 360.0;
        let rgb = hsv_to_rgb(self.hue, 1.0, 1.0);
        let mut data = Vec::with_capacity((leds * 3) as usize);
        for _ in 0..leds { data.extend_from_slice(&rgb); }
        data
    }
}

// --- The Scan Effect ---
pub struct ScanEffect { pub position: u32, pub color: [u8; 3] }
impl Effect for ScanEffect {
    fn render_frame(&mut self, leds: u32, _audio_data: &AudioAnalysisData) -> Vec<u8> {
        self.position = (self.position + 1) % leds;
        let mut data = vec![0; (leds * 3) as usize];
        let start_index = (self.position * 3) as usize;
        if start_index + 3 <= data.len() { data[start_index..start_index + 3].copy_from_slice(&self.color); }
        data
    }
}

// --- The Scroll Effect ---
pub struct ScrollEffect { pub hue: f32 }
impl Effect for ScrollEffect {
    fn render_frame(&mut self, leds: u32, _audio_data: &AudioAnalysisData) -> Vec<u8> {
        self.hue = (self.hue + 0.5) % 360.0;
        let mut data = Vec::with_capacity((leds * 3) as usize);
        for i in 0..leds {
            let pixel_hue = (self.hue + (i as f32 * 10.0)) % 360.0;
            let rgb = hsv_to_rgb(pixel_hue, 1.0, 1.0);
            data.extend_from_slice(&rgb);
        }
        data
    }
}

// --- The BladePower Effect ---
pub struct BladePowerEffect;
impl Effect for BladePowerEffect {
    fn render_frame(&mut self, leds: u32, audio_data: &AudioAnalysisData) -> Vec<u8> {
        let mut data = vec![0; (leds * 3) as usize];
        let power_leds = (leds as f32 * audio_data.volume) as u32;

        for i in 0..power_leds {
            let hue = 120.0 * (1.0 - audio_data.volume); // Green to Red
            let rgb = hsv_to_rgb(hue, 1.0, 1.0);
            let start_index = (i * 3) as usize;
            data[start_index..start_index + 3].copy_from_slice(&rgb);
        }
        data
    }
}