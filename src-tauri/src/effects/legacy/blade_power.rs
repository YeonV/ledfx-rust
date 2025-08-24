use crate::effects::{BaseEffectConfig, Effect};
use crate::utils::colors::{self};
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

#[derive(Deserialize, Serialize, Type, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct BladePowerLegacyConfig {
    pub decay: f32,
    pub multiplier: f32,
    pub frequency_range: String,
    pub gradient: String,
    #[serde(flatten)]
    pub base: BaseEffectConfig,
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
        self.gradient_palette = colors::parse_gradient(&self.config.gradient, pixel_count);
    }
}

impl Effect for BladePowerLegacy {
    fn render(&mut self, audio_data: &AudioAnalysisData, frame: &mut [u8]) {
        let pixel_count = frame.len() / 3;
        if pixel_count == 0 { return; }

        if self.gradient_palette.len() != pixel_count {
            self.rebuild_palette(pixel_count);
        }

        let power = match self.config.frequency_range.as_str() {
            "Mids" => mids_power(&audio_data.melbanks),
            "High" => highs_power(&audio_data.melbanks),
            _ => lows_power(&audio_data.melbanks),
        };
        
        let bar_level = (power * self.config.multiplier * 2.0).min(1.0);
        let bar_idx = (bar_level * pixel_count as f32) as usize;

        let decay_factor = self.config.decay / 2.0 + 0.45;
        for v in self.v_channel.iter_mut() {
            *v *= decay_factor;
        }

        for i in 0..bar_idx {
            self.v_channel[i] = 1.0;
        }

        for i in 0..pixel_count {
            let base_color = self.gradient_palette[i];
            let brightness = self.v_channel[i];

            let r = (base_color[0] as f32 * brightness) as u8;
            let g = (base_color[1] as f32 * brightness) as u8;
            let b = (base_color[2] as f32 * brightness) as u8;

            frame[i * 3]     = r;
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
    
    fn get_base_config(&self) -> BaseEffectConfig {
        self.config.base.clone()
    }
}