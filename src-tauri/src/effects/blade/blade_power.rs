// src-tauri/src/effects/blade/blade_power.rs

use crate::audio::AudioAnalysisData;
use crate::effects::Effect;
use crate::utils::hsv_to_rgb;
use serde::{Deserialize, Serialize};
use specta::Type;

// --- Composable Config Structs ---
#[derive(Deserialize, Serialize, Type, Clone)]
pub struct BaseEffectConfig {
    pub brightness: f32,
    pub blur: f32,
    pub mirror: bool,
    pub flip: bool,
}

#[derive(Deserialize, Serialize, Type, Clone)]
pub struct AudioReactiveConfig {
    pub frequency_range: String,
}

// The final, clean config for this effect.
#[derive(Deserialize, Serialize, Type, Clone)]
pub struct BladePowerConfig {
    pub base: BaseEffectConfig,
    pub audio: AudioReactiveConfig,
    pub decay: f32,
    pub sensitivity: f32,
}

pub struct BladePower {
    config: BladePowerConfig,
    hsv_array: Vec<(f32, f32, f32)>,
    bar_level: f32,
    pixel_count: u32,
}

impl BladePower {
    pub fn new(config: BladePowerConfig, pixel_count: u32) -> Self {
        let mut hsv_array = vec![(0.0, 0.0, 0.0); pixel_count as usize];
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

impl Effect for BladePower {
    // --- THE FIX: The signature now matches the trait ---
    fn render_frame(&mut self, pixel_count: u32, audio_data: &AudioAnalysisData) -> Vec<u8> {
        // Safety check
        if pixel_count != self.pixel_count {
            return vec![0; (pixel_count * 3) as usize];
        }

        // The rest of your logic is correct.
        self.bar_level = (audio_data.volume * self.config.sensitivity).min(1.0);

        let bar_idx = (self.bar_level * self.pixel_count as f32) as usize;

        for i in 0..self.pixel_count as usize {
            self.hsv_array[i].2 *= self.config.decay;
        }
        for i in 0..bar_idx {
            self.hsv_array[i].2 = self.config.base.brightness;
        }

        let mut rgb_pixels = Vec::with_capacity((self.pixel_count * 3) as usize);
        for (h, s, v) in &self.hsv_array {
            let rgb = hsv_to_rgb(*h, *s, *v);
            rgb_pixels.extend_from_slice(&rgb);
        }

        rgb_pixels
    }
    
    fn update_settings(&mut self, settings: serde_json::Value) {
        if let Ok(new_config) = serde_json::from_value(settings) {
            self.config = new_config;
        } else {
            eprintln!("Failed to deserialize settings for BladePower");
        }
    }
}