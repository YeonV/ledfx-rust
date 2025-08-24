use crate::audio::{lows_power, AudioAnalysisData};
use crate::effects::{BaseEffectConfig, Effect}; // Add BaseEffectConfig
use crate::utils::colors::hsv_to_rgb;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;

// --- START: CONFIG REFACTOR ---
#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct BladePowerConfig {
    pub sensitivity: f32,

    #[serde(flatten)]
    pub base: BaseEffectConfig,
}
// --- END: CONFIG REFACTOR ---

pub struct BladePower {
    config: BladePowerConfig,
    bar_level: f32,
}

impl BladePower {
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

        let power = lows_power(&audio_data.melbanks);
        // The decay logic here is simpler than legacy, we can refine it later
        self.bar_level = (self.bar_level * 0.95).max(power * self.config.sensitivity);

        let bar_pixels = (self.bar_level * pixel_count as f32) as usize;

        for i in 0..pixel_count {
            let rgb = if i < bar_pixels {
                hsv_to_rgb(0.0, 1.0, 1.0) // Red
            } else {
                [0, 0, 0] // Off
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

    // --- START: IMPLEMENT NEW TRAIT METHOD ---
    fn get_base_config(&self) -> BaseEffectConfig {
        self.config.base.clone()
    }
    // --- END: IMPLEMENT NEW TRAIT METHOD ---
}