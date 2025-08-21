// src-tauri/src/effects/legacy/blade_power.rs
// src-tauri/src/effects/legacy/blade_power.rs

use crate::audio::AudioAnalysisData;
use crate::effects::Effect;
use serde::{Deserialize, Serialize};
use specta::Type;
use crate::utils::parse_gradient; 

#[derive(Serialize, Type, Clone)]
#[serde(untagged)]
pub enum DefaultValue {
    String(String),
    Number(f32),
    Bool(bool),
}

// --- THE FIX: A consistent, tagged enum for all controls ---
#[derive(Serialize, Type, Clone)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Control {
    Slider { min: f32, max: f32, step: f32 },
    Checkbox,
    ColorPicker,
    Select { options: Vec<String> },
}

#[derive(Serialize, Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EffectSetting {
    pub id: String,
    pub name: String,
    pub description: String,
    pub control: Control,
    pub default_value: DefaultValue,
}

#[derive(Deserialize, Serialize, Type, Clone)]
#[serde(rename_all = "snake_case")]
pub struct BladePowerLegacyConfig {
    pub mirror: bool,
    pub blur: f32,
    pub decay: f32,
    pub multiplier: f32,
    pub background_color: String,
    pub frequency_range: String,
    pub gradient: String,
}

pub fn get_schema() -> Vec<EffectSetting> {
    vec![
        EffectSetting {
            id: "mirror".to_string(),
            name: "Mirror".to_string(),
            description: "Mirror the effect".to_string(),
            control: Control::Checkbox, // No change needed here
            default_value: DefaultValue::Bool(false),
        },
        EffectSetting {
            id: "blur".to_string(),
            name: "Blur".to_string(),
            description: "Amount to blur the effect".to_string(),
            control: Control::Slider { min: 0.0, max: 10.0, step: 0.1 },
            default_value: DefaultValue::Number(2.0),
        },
        EffectSetting {
            id: "decay".to_string(),
            name: "Decay".to_string(),
            description: "Rate of color decay".to_string(),
            control: Control::Slider { min: 0.0, max: 1.0, step: 0.01 },
            default_value: DefaultValue::Number(0.7),
        },
        EffectSetting {
            id: "multiplier".to_string(),
            name: "Multiplier".to_string(),
            description: "Make the reactive bar bigger/smaller".to_string(),
            control: Control::Slider { min: 0.0, max: 1.0, step: 0.01 },
            default_value: DefaultValue::Number(0.5),
        },
        EffectSetting {
            id: "background_color".to_string(),
            name: "Background Color".to_string(),
            description: "Color of Background".to_string(),
            control: Control::ColorPicker, // No change needed here
            default_value: DefaultValue::String("#000000".to_string()),
        },
        EffectSetting {
            id: "frequency_range".to_string(),
            name: "Frequency Range".to_string(),
            description: "Frequency range for the beat detection".to_string(),
            control: Control::Select {
                options: vec![
                    "Lows (beat+bass)".to_string(),
                    "Mids".to_string(),
                    "High".to_string(),
                    "Bass".to_string(),
                ],
            },
            default_value: DefaultValue::String("Lows (beat+bass)".to_string()),
        },
        EffectSetting {
            id: "gradient".to_string(),
            name: "Gradient".to_string(),
            description: "Color gradient for the effect".to_string(),
            control: Control::ColorPicker, // A gradient picker would go here
            default_value: DefaultValue::String("linear-gradient(90deg, #ff0000 0%, #0000ff 100%)".to_string()),
        },
    ]
}

pub struct BladePowerLegacy {
    pub config: BladePowerLegacyConfig,
    pixel_count: u32,
    gradient_palette: Vec<[u8; 3]>,
    v_channel: Vec<f32>, // The brightness (Value) for each pixel
}

impl BladePowerLegacy {
    pub fn new(config: BladePowerLegacyConfig, pixel_count: u32) -> Self {
        let gradient_palette = parse_gradient(&config.gradient, pixel_count as usize);
        
        Self {
            config,
            pixel_count,
            gradient_palette,
            v_channel: vec![0.0; pixel_count as usize], // Initialize all pixels to black
        }
    }
}

impl Effect for BladePowerLegacy {
    fn render_frame(&mut self, audio_data: &AudioAnalysisData) -> Vec<u8> {
        // --- THE FIX: The correct, stateful rendering logic ---
        let power = audio_data.volume;
        let bar_level = (power * self.config.multiplier * 2.0).min(1.0);
        let bar_idx = (bar_level * self.pixel_count as f32) as usize;

        // 1. Apply decay to all pixels
        for v in self.v_channel.iter_mut() {
            *v *= self.config.decay / 2.0 + 0.45;
        }

        // 2. Apply new power to the active part of the bar
        for i in 0..bar_idx {
            self.v_channel[i] = 1.0; // Set to full brightness
        }

        // 3. Generate the final RGB pixels
        let mut rgb_pixels = Vec::with_capacity((self.pixel_count * 3) as usize);
        for i in 0..self.pixel_count as usize {
            let base_color = self.gradient_palette[i];
            let brightness = self.v_channel[i];

            // Multiply the gradient color by the current brightness value
            let r = (base_color[0] as f32 * brightness) as u8;
            let g = (base_color[1] as f32 * brightness) as u8;
            let b = (base_color[2] as f32 * brightness) as u8;
            
            rgb_pixels.extend_from_slice(&[r, g, b]);
        }
        
        rgb_pixels
    }
     fn update_settings(&mut self, settings: serde_json::Value) {
        if let Ok(new_config) = serde_json::from_value(settings) {
            self.config = new_config;
            // If the gradient changes, we must re-calculate the palette.
            self.gradient_palette = parse_gradient(&self.config.gradient, self.pixel_count as usize);
        } else {
            eprintln!("Failed to deserialize settings for BladePowerLegacy");
        }
    }
}
