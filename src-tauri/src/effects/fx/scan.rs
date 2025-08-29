use crate::audio::AudioAnalysisData;
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

pub const NAME: &str = "Scan";

#[derive(Deserialize, Serialize, Type, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ScanConfig {
    pub speed: f32,
    pub width: f32,
    pub gradient: String,
    #[serde(flatten)]
    pub base: BaseEffectConfig,
}

pub fn get_schema() -> Vec<EffectSetting> {
    let mut schema = get_base_schema();
    schema.extend(vec![
        EffectSetting {
            id: "speed".to_string(),
            name: "Speed".to_string(),
            description: "Speed of the scanner".to_string(),
            control: Control::Slider {
                min: 1.0,
                max: 10.0,
                step: 0.1,
            },
            default_value: DefaultValue::Number(1.0),
        },
        EffectSetting {
            id: "width".to_string(),
            name: "Width".to_string(),
            description: "Width of the scanner".to_string(),
            control: Control::Slider {
                min: 1.0,
                max: 50.0,
                step: 1.0,
            },
            default_value: DefaultValue::Number(10.0),
        },
        EffectSetting {
            id: "gradient".to_string(),
            name: "Gradient".to_string(),
            description: "Color gradient for the scanner".to_string(),
            control: Control::ColorPicker,
            default_value: DefaultValue::String(
                "linear-gradient(90deg, #ff0000 0%, #00ff00 100%)".to_string(),
            ),
        },
    ]);
    schema
}

pub struct Scan {
    pub config: ScanConfig,
    gradient_palette: Vec<[u8; 3]>,
    position: f32,
}

impl Scan {
    pub fn new(config: ScanConfig) -> Self {
        Self {
            config,
            gradient_palette: Vec::new(),
            position: 0.0,
        }
    }

    fn rebuild_palette(&mut self) {
        let palette_size = self.config.width.ceil() as usize;
        self.gradient_palette = colors::parse_gradient(&self.config.gradient, palette_size);
    }
}

impl Effect for Scan {
    fn render(&mut self, _audio_data: &AudioAnalysisData, frame: &mut [u8]) {
        let pixel_count = frame.len() / 3;
        if pixel_count == 0 {
            return;
        }

        let width = self.config.width.ceil() as usize;
        if self.gradient_palette.len() != width {
            self.rebuild_palette();
        }
        if self.gradient_palette.is_empty() {
            return;
        }

        self.position = (self.position + self.config.speed) % (pixel_count as f32);

        frame.fill(0);

        let start_pixel = self.position.floor() as usize;
        for i in 0..width {
            let pixel_index = (start_pixel + i) % pixel_count;
            let color_index = i % self.gradient_palette.len();

            let color = self.gradient_palette[color_index];
            let frame_index = pixel_index * 3;

            if frame_index + 2 < frame.len() {
                frame[frame_index] = color[0];
                frame[frame_index + 1] = color[1];
                frame[frame_index + 2] = color[2];
            }
        }
    }

    fn update_config(&mut self, config: Value) {
        if let Ok(new_config) = serde_json::from_value(config) {
            self.config = new_config;
            self.gradient_palette.clear();
        } else {
            eprintln!("Failed to deserialize settings for Scan");
        }
    }

    fn get_base_config(&self) -> BaseEffectConfig {
        self.config.base.clone()
    }
}

use std::collections::HashMap;
// This function must exist to satisfy the generated code.
// It can be empty if there are no built-in presets for this effect.
pub fn get_built_in_presets() -> HashMap<String, EffectConfig> {
    let mut presets = HashMap::new();

    presets.insert(
        "K.I.T.T.".to_string(),
        EffectConfig::Scan(ScanConfig {
            speed: 5.0,
            width: 0.15,
            gradient: "linear-gradient(90deg, #ff0000 0%, #330000 50%, #000000 100%)".to_string(),
            base: BaseEffectConfig {
                mirror: true,
                flip: false,
                blur: 2.0,
                background_color: "#000000".to_string(),
            },
        }),
    );

    presets.insert(
        "Cylon".to_string(),
        EffectConfig::Scan(ScanConfig {
            speed: 2.0,
            width: 0.05,
            gradient: "linear-gradient(90deg, #ff0000 0%, #000000 100%)".to_string(),
            base: BaseEffectConfig {
                mirror: true,
                flip: false,
                blur: 0.5,
                background_color: "#110000".to_string(),
            },
        }),
    );

    presets.insert(
        "Rainbow Chase".to_string(),
        EffectConfig::Scan(ScanConfig {
            speed: 8.0,
            width: 0.5,
            gradient:
                "linear-gradient(90deg, #ff0000, #ffff00, #00ff00, #00ffff, #0000ff, #ff00ff)"
                    .to_string(),
            base: BaseEffectConfig {
                mirror: false,
                flip: false,
                blur: 0.0,
                background_color: "#000000".to_string(),
            },
        }),
    );

    presets
}
