use crate::effects::Effect;
use crate::utils::colors::parse_gradient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use crate::audio::{highs_power, lows_power, mids_power, AudioAnalysisData};

#[derive(Serialize, Type, Clone)]
#[serde(untagged)]
pub enum DefaultValue {
    String(String),
    Number(f32),
    Bool(bool),
}

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
            control: Control::Checkbox,
            default_value: DefaultValue::Bool(false),
        },
        EffectSetting {
            id: "blur".to_string(),
            name: "Blur".to_string(),
            description: "Amount to blur the effect".to_string(),
            control: Control::Slider {
                min: 0.0,
                max: 10.0,
                step: 0.1,
            },
            default_value: DefaultValue::Number(2.0),
        },
        EffectSetting {
            id: "decay".to_string(),
            name: "Decay".to_string(),
            description: "Rate of color decay".to_string(),
            control: Control::Slider {
                min: 0.0,
                max: 1.0,
                step: 0.01,
            },
            default_value: DefaultValue::Number(0.7),
        },
        EffectSetting {
            id: "multiplier".to_string(),
            name: "Multiplier".to_string(),
            description: "Make the reactive bar bigger/smaller".to_string(),
            control: Control::Slider {
                min: 0.0,
                max: 1.0,
                step: 0.01,
            },
            default_value: DefaultValue::Number(0.5),
        },
        EffectSetting {
            id: "background_color".to_string(),
            name: "Background Color".to_string(),
            description: "Color of Background".to_string(),
            control: Control::ColorPicker,
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
                ],
            },
            default_value: DefaultValue::String("Lows (beat+bass)".to_string()),
        },
        EffectSetting {
            id: "gradient".to_string(),
            name: "Gradient".to_string(),
            description: "Color gradient for the effect".to_string(),
            control: Control::ColorPicker,
            default_value: DefaultValue::String(
                "linear-gradient(90deg, #ff0000 0%, #0000ff 100%)".to_string(),
            ),
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
    gradient_palette: Vec<[u8; 3]>,
    v_channel: Vec<f32>,
}

impl BladePowerLegacy {
    pub fn new(config: BladePowerLegacyConfig) -> Self {
        Self {
            config,
            gradient_palette: Vec::new(),
            v_channel: Vec::new(),
        }
    }

    fn rebuild_palette(&mut self, pixel_count: usize) {
        if pixel_count == 0 { return; }
        if self.v_channel.len() != pixel_count {
            self.v_channel = vec![0.0; pixel_count];
        }

        let mut base_palette = parse_gradient(&self.config.gradient, pixel_count);

        if self.config.flip {
            base_palette.reverse();
        }

        if self.config.mirror {
            let half_len = (pixel_count as f32 / 2.0).ceil() as usize;
            let squeezed_palette = resample_palette(&base_palette, half_len);
            let mut final_palette = vec![[0, 0, 0]; pixel_count];

            for i in 0..half_len {
                let color = squeezed_palette[i];
                final_palette[i] = color;
                if (pixel_count - 1 - i) < final_palette.len() {
                    final_palette[pixel_count - 1 - i] = color;
                }
            }
            self.gradient_palette = final_palette;
        } else {
            self.gradient_palette = base_palette;
        }
    }
}

fn resample_palette(palette: &[[u8; 3]], new_len: usize) -> Vec<[u8; 3]> {
    if palette.is_empty() || new_len == 0 {
        return Vec::new();
    }
    let mut new_palette = Vec::with_capacity(new_len);
    let old_len = palette.len();
    for i in 0..new_len {
        let old_index =
            (i as f32 * (old_len - 1) as f32 / (new_len - 1).max(1) as f32).round() as usize;
        new_palette.push(palette[old_index]);
    }
    new_palette
}

impl Effect for BladePowerLegacy {
    fn render(&mut self, audio_data: &AudioAnalysisData, frame: &mut [u8]) {
        let pixel_count = frame.len() / 3;
        if pixel_count == 0 { return; }

        if self.gradient_palette.len() != pixel_count || self.v_channel.len() != pixel_count {
            self.rebuild_palette(pixel_count);
        }

        // --- THIS IS THE FIX ---
        let power = match self.config.frequency_range.as_str() {
            "Mids" => mids_power(&audio_data.melbanks),
            "High" => highs_power(&audio_data.melbanks),
            _ => lows_power(&audio_data.melbanks),
        };

        
        let bar_level = (power * self.config.multiplier * 2.0).min(1.0);

        let bar_idx = if self.config.mirror {
            (bar_level * (pixel_count as f32 / 2.0)) as usize
        } else {
            (bar_level * pixel_count as f32) as usize
        };

        let decay_factor = self.config.decay / 2.0 + 0.45;
        for v in self.v_channel.iter_mut() {
            *v *= decay_factor;
        }

        if self.config.mirror {
            for i in 0..bar_idx {
                self.v_channel[i] = 1.0;
                if (pixel_count - 1 - i) < self.v_channel.len() {
                    self.v_channel[pixel_count - 1 - i] = 1.0;
                }
            }
        } else {
            for i in 0..bar_idx {
                self.v_channel[i] = 1.0;
            }
        }

        for i in 0..pixel_count {
            let base_color = self.gradient_palette[i];
            let brightness = self.v_channel[i];

            let r = (base_color[0] as f32 * brightness) as u8;
            let g = (base_color[1] as f32 * brightness) as u8;
            let b = (base_color[2] as f32 * brightness) as u8;

            frame[i * 3] = r;
            frame[i * 3 + 1] = g;
            frame[i * 3 + 2] = b;
        }
    }

    fn update_config(&mut self, config: Value) {
        if let Ok(new_config) = serde_json::from_value(config) {
            self.config = new_config;
            self.gradient_palette.clear();
        } else {
            eprintln!("Failed to deserialize settings for BladePowerLegacy");
        }
    }
}