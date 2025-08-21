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
    pub flip: bool,
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
        EffectSetting {
            id: "flip".to_string(),
            name: "Flip".to_string(),
            description: "Flip the effect direction".to_string(),
            control: Control::Checkbox,
            default_value: DefaultValue::Bool(false),
        },
    ]
}

pub struct BladePowerLegacy {
    pub config: BladePowerLegacyConfig,
    pixel_count: u32,
    gradient_palette: Vec<[u8; 3]>,
    v_channel: Vec<f32>,
}

impl BladePowerLegacy {
    pub fn new(config: BladePowerLegacyConfig, pixel_count: u32) -> Self {
        let mut instance = Self {
            config,
            pixel_count,
            gradient_palette: Vec::new(),
            v_channel: vec![0.0; pixel_count as usize],
        };
        instance.rebuild_palette();
        instance
    }

    fn rebuild_palette(&mut self) {
        // --- THE FIX: The correct order of operations ---

        // 1. Parse the full gradient.
        let mut base_palette = parse_gradient(&self.config.gradient, self.pixel_count as usize);

        // 2. Flip the full gradient first if flip is on.
        if self.config.flip {
            base_palette.reverse();
        }

        if self.config.mirror {
            let half_len = (self.pixel_count as f32 / 2.0).ceil() as usize;
            
            // 3. Squeeze the (potentially flipped) base palette into the first half.
            let squeezed_palette = resample_palette(&base_palette, half_len);

            let mut final_palette = vec![[0, 0, 0]; self.pixel_count as usize];
            
            // 4. Build the final mirrored palette from the squeezed version.
            for i in 0..half_len {
                let color = squeezed_palette[i];
                final_palette[i] = color;
                final_palette[self.pixel_count as usize - 1 - i] = color;
            }
            
            self.gradient_palette = final_palette;
        } else {
            // If not mirroring, just use the (potentially flipped) base palette.
            self.gradient_palette = base_palette;
        }
    }
}

// --- THE FIX: This function now correctly returns the new palette ---
fn resample_palette(palette: &[[u8; 3]], new_len: usize) -> Vec<[u8; 3]> {
    if palette.is_empty() || new_len == 0 {
        return Vec::new();
    }
    let mut new_palette = Vec::with_capacity(new_len);
    let old_len = palette.len();
    for i in 0..new_len {
        let old_index = (i as f32 * (old_len - 1) as f32 / (new_len - 1).max(1) as f32).round() as usize;
        new_palette.push(palette[old_index]);
    }
    new_palette
}

impl Effect for BladePowerLegacy {
    fn render_frame(&mut self, _pixel_count: u32, audio_data: &AudioAnalysisData) -> Vec<u8> {
        let bar_level = (audio_data.volume * self.config.multiplier * 2.0).min(1.0);
        
        let bar_idx = if self.config.mirror {
            (bar_level * (self.pixel_count as f32 / 2.0)) as usize
        } else {
            (bar_level * self.pixel_count as f32) as usize
        };

        let decay_factor = self.config.decay / 2.0 + 0.45;
        for v in self.v_channel.iter_mut() {
            *v *= decay_factor;
        }

        if self.config.mirror {
            for i in 0..bar_idx {
                self.v_channel[i] = 1.0;
                self.v_channel[self.pixel_count as usize - 1 - i] = 1.0;
            }
        } else {
            for i in 0..bar_idx {
                self.v_channel[i] = 1.0;
            }
        }

        let mut rgb_pixels = Vec::with_capacity((self.pixel_count * 3) as usize);
        for i in 0..self.pixel_count as usize {
            let base_color = self.gradient_palette[i];
            let brightness = self.v_channel[i];
            
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
            self.rebuild_palette();
        } else {
            eprintln!("Failed to deserialize settings for BladePowerLegacy");
        }
    }
}