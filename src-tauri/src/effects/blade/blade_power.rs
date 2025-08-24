use crate::audio::{lows_power, AudioAnalysisData}; // Add lows_power to imports
use crate::effects::Effect;
use crate::utils::colors::hsv_to_rgb;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone)]
pub struct BladePowerConfig {
    pub sensitivity: f32,
}

pub struct BladePower {
    config: BladePowerConfig,
    bar_level: f32,
}

impl BladePower {
    // FIX: Remove redundant pixel_count
    pub fn new(config: BladePowerConfig) -> Self {
        Self {
            config,
            bar_level: 0.0,
        }
    }
}

impl Effect for BladePower {
    fn render(&mut self, audio_data: &AudioAnalysisData, frame: &mut [u8]) {
        let pixel_count = frame.len() / 3;

        // --- THIS IS THE FIX ---
        // Call `lows_power` as a function, passing the melbanks slice.
        let power = lows_power(&audio_data.melbanks);
        // --- END FIX ---

        self.bar_level = (power * self.config.sensitivity).min(1.0);

        let bar_pixels = (self.bar_level * pixel_count as f32) as usize;

        for i in 0..pixel_count {
            let rgb = if i < bar_pixels {
                hsv_to_rgb(0.0, 1.0, 1.0)
            } else {
                [0, 0, 0]
            };
            frame[i * 3] = rgb[0];
            frame[i * 3 + 1] = rgb[1];
            frame[i * 3 + 2] = rgb[2];
        }
    }

    fn update_config(&mut self, config: Value) {
        if let Ok(new_config) = serde_json::from_value(config) {
            self.config = new_config;
        }
    }
}