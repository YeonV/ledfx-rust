// src-tauri/src/effects/legacy/blade_power.rs

use crate::audio::AudioAnalysisData;
use crate::effects::Effect;
use crate::utils::hsv_to_rgb;
use serde::Deserialize;
use specta::Type;

// --- 1:1 Config Struct ---
// This struct exactly mirrors the Python CONFIG_SCHEMA.
#[derive(Deserialize, Type, Clone)]
#[serde(rename_all = "snake_case")]
pub struct BladePowerLegacyConfig {
    pub mirror: bool,
    pub blur: f32,
    pub decay: f32,
    pub multiplier: f32,
    pub background_color: String, // Assuming color comes as hex string
    pub frequency_range: String, // "Lows (beat+bass)", etc.
}

pub struct BladePowerLegacy {
    config: BladePowerLegacyConfig,
    hsv_array: Vec<(f32, f32, f32)>, // (h, s, v)
    bar_level: f32,
    pixel_count: u32,
}

impl BladePowerLegacy {
    pub fn new(config: BladePowerLegacyConfig, pixel_count: u32) -> Self {
        let mut hsv_array = vec![(0.0, 0.0, 0.0); pixel_count as usize];
        // Pre-calculate the hue and saturation gradient on activation.
        for i in 0..pixel_count {
            hsv_array[i as usize] = (i as f32 / pixel_count as f32, 1.0, 0.0);
        }

        Self {
            config,
            hsv_array,
            bar_level: 0.0,
            pixel_count,
        }
    }
}

impl Effect for BladePowerLegacy {
    fn render_frame(&mut self, audio_data: &AudioAnalysisData) -> Vec<u8> {
        // --- Audio Update Logic ---
        // For now, we only have 'volume'. We will replace this with multi-band data later.
        self.bar_level = (audio_data.volume * self.config.multiplier * 2.0).min(1.0);

        // --- Render HSV Logic ---
        let bar_idx = (self.bar_level * self.pixel_count as f32) as usize;

        for i in 0..self.pixel_count as usize {
            // Apply decay
            self.hsv_array[i].2 *= self.config.decay / 2.0 + 0.45;
        }
        for i in 0..bar_idx {
            // Apply power
            self.hsv_array[i].2 = 1.0; // We will use a base brightness setting later
        }

        // --- Convert to RGB ---
        let mut rgb_pixels = Vec::with_capacity((self.pixel_count * 3) as usize);
        for (h, s, v) in &self.hsv_array {
            let rgb = hsv_to_rgb(*h, *s, *v);
            rgb_pixels.extend_from_slice(&rgb);
        }

        rgb_pixels
    }
}