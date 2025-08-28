use crate::effects::{get_base_schema, BaseEffectConfig, Effect, schema::{Control, DefaultValue, EffectSetting}};
use crate::utils::colors;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use crate::audio::AudioAnalysisData;
use rand::Rng;
use crate::engine::EffectConfig;

pub const NAME: &str = "Fire";

#[derive(Deserialize, Serialize, Type, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct FireConfig {
    pub cooling: f32,
    pub sparking: f32,
    pub gradient: String,
    
    #[serde(flatten)]
    pub base: BaseEffectConfig,
}

pub fn get_schema() -> Vec<EffectSetting> {
    let mut schema = get_base_schema();
    schema.extend(vec![
        EffectSetting {
            id: "cooling".to_string(),
            name: "Cooling".to_string(),
            description: "How fast the fire cools down and fades".to_string(),
            control: Control::Slider { min: 20.0, max: 100.0, step: 1.0 },
            default_value: DefaultValue::Number(55.0),
        },
        EffectSetting {
            id: "sparking".to_string(),
            name: "Sparking".to_string(),
            description: "The brightness and frequency of new sparks".to_string(),
            control: Control::Slider { min: 50.0, max: 200.0, step: 1.0 },
            default_value: DefaultValue::Number(120.0),
        },
        EffectSetting {
            id: "gradient".to_string(),
            name: "Gradient".to_string(),
            description: "Color gradient for the fire".to_string(),
            control: Control::ColorPicker,
            default_value: DefaultValue::String(
                "linear-gradient(90deg, #000000 0%, #ff0000 50%, #ffff00 100%)".to_string(),
            ),
        },
    ]);
    schema
}

pub struct Fire {
    pub config: FireConfig,
    gradient_palette: Vec<[u8; 3]>,
    heat: Vec<u8>,
}

impl Fire {
    pub fn new(config: FireConfig) -> Self {
        Self {
            config,
            gradient_palette: Vec::new(),
            heat: Vec::new(),
        }
    }

    fn rebuild_palette(&mut self) {
        // We generate a 256-color palette to map heat values (0-255) to colors
        self.gradient_palette = colors::parse_gradient(&self.config.gradient, 256);
    }
}

impl Effect for Fire {
    fn render(&mut self, _audio_data: &AudioAnalysisData, frame: &mut [u8]) {
        let pixel_count = frame.len() / 3;
        if pixel_count == 0 { return; }

        if self.heat.len() != pixel_count {
            self.heat = vec![0; pixel_count];
        }
        if self.gradient_palette.is_empty() {
            self.rebuild_palette();
        }

        // 1. Cool down every cell
        let mut rng = rand::thread_rng();
        for i in 0..pixel_count {
            let cooldown = rng.gen_range(0..=((self.config.cooling * 10.0 / pixel_count as f32) as u32 + 2));
            self.heat[i] = self.heat[i].saturating_sub(cooldown as u8);
        }

        // 2. Propagate heat upwards
        for i in (3..pixel_count).rev() {
            self.heat[i] = ((self.heat[i - 1] as u16 + self.heat[i - 2] as u16 + self.heat[i - 2] as u16) / 3) as u8;
        }

        // 3. Add new sparks at the bottom
        let spark_heat = (rng.gen_range(0.0..=255.0) * self.config.sparking / 255.0) as u8;
        if spark_heat > self.heat[0] {
            self.heat[0] = spark_heat;
        }

        // 4. Map heat to color
        for i in 0..pixel_count {
            let color_index = self.heat[i] as usize;
            let color = self.gradient_palette[color_index];
            frame[i * 3] = color[0];
            frame[i * 3 + 1] = color[1];
            frame[i * 3 + 2] = color[2];
        }
    }

    fn update_config(&mut self, config: Value) {
        if let Ok(new_config) = serde_json::from_value(config) {
            self.config = new_config;
            self.gradient_palette.clear();
        } else {
            eprintln!("Failed to deserialize settings for Fire");
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
        "Classic Campfire".to_string(),
        EffectConfig::Fire(FireConfig {
            cooling: 0.45,
            sparking: 0.6,
            gradient: "linear-gradient(90deg, #000000 0%, #D43300 30%, #FF8000 70%, #FFFF00 100%)".to_string(),
            base: BaseEffectConfig { mirror: false, flip: false, blur: 1.5, background_color: "#000000".to_string() },
        }),
    );

    presets.insert(
        "Soul Fire".to_string(),
        EffectConfig::Fire(FireConfig {
            cooling: 0.6,
            sparking: 0.4,
            gradient: "linear-gradient(90deg, #000000 0%, #00FFFF 50%, #FFFFFF 100%)".to_string(),
            base: BaseEffectConfig { mirror: false, flip: false, blur: 2.0, background_color: "#000000".to_string() },
        }),
    );

    presets.insert(
        "Nuclear Waste".to_string(),
        EffectConfig::Fire(FireConfig {
            cooling: 0.3,
            sparking: 0.8,
            gradient: "linear-gradient(90deg, #000000 0%, #00FF00 40%, #ADFF2F 100%)".to_string(),
            base: BaseEffectConfig { mirror: true, flip: false, blur: 0.5, background_color: "#000000".to_string() },
        }),
    );
    
    presets
}