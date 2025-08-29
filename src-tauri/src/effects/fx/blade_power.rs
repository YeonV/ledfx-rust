use crate::audio::{highs_power, lows_power, mids_power, AudioAnalysisData};
use crate::effects::{
    get_base_schema,
    schema::{Control, DefaultValue, EffectSetting},
    BaseEffectConfig, Effect,
};
use crate::engine::EffectConfig;
use crate::utils::colors;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use std::collections::HashMap;

pub const NAME: &str = "Blade Power";

#[derive(Deserialize, Serialize, Type, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct BladePowerConfig {
    pub decay: f32,
    pub multiplier: f32,
    pub frequency_range: String,
    pub gradient: String,
    #[serde(flatten)]
    pub base: BaseEffectConfig,
}

pub fn get_schema() -> Vec<EffectSetting> {
    let mut schema = get_base_schema();
    schema.extend(vec![
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
    ]);
    schema
}

pub struct BladePower {
    pub config: BladePowerConfig,
    gradient_palette: Vec<[u8; 3]>,
    v_channel: Vec<f32>,
}

impl BladePower {
    pub fn new(config: BladePowerConfig) -> Self {
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

impl Effect for BladePower {
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
            eprintln!("Failed to deserialize settings for BladePower");
        }
    }

    fn get_base_config(&self) -> BaseEffectConfig {
        self.config.base.clone()
    }
}

pub fn get_built_in_presets() -> HashMap<String, EffectConfig> {
    let mut presets = HashMap::new();

    presets.insert(
        "Ocean Breeze".to_string(),
        EffectConfig::BladePower(BladePowerConfig {
            decay: 0.8,
            multiplier: 0.6,
            frequency_range: "Lows (beat+bass)".to_string(),
            gradient: "linear-gradient(90deg, #00c6ff 0%, #0072ff 100%)".to_string(),
            base: BaseEffectConfig { mirror: false, flip: false, blur: 1.0, background_color: "#000000".to_string() },
        }),
    );

    presets.insert(
        "Sunset".to_string(),
        EffectConfig::BladePower(BladePowerConfig {
            decay: 0.75,
            multiplier: 0.5,
            frequency_range: "Mids".to_string(),
            gradient: "linear-gradient(90deg, #ff4e50 0%, #f9d423 100%)".to_string(),
            base: BaseEffectConfig { mirror: true, flip: false, blur: 0.0, background_color: "#000000".to_string() },
        }),
    );

    presets.insert(
        "Forrest Gump".to_string(),
        EffectConfig::BladePower(BladePowerConfig {
            decay: 0.6,
            multiplier: 0.7,
            frequency_range: "Lows (beat+bass)".to_string(),
            gradient: "linear-gradient(90deg, #1d976c 0%, #93f9b9 100%)".to_string(),
            base: BaseEffectConfig { mirror: false, flip: true, blur: 2.0, background_color: "#050505".to_string() },
        }),
    );

    presets
}