use crate::audio::AudioAnalysisData;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use crate::effects::blade_power::{Control, DefaultValue, EffectSetting};

pub mod blade_power;
pub mod scan;

#[derive(Deserialize, Serialize, Type, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct BaseEffectConfig {
    pub mirror: bool,
    pub flip: bool,
    pub blur: f32,
    pub background_color: String,
}

pub trait Effect: Send + Sync {
    fn render(&mut self, audio_data: &AudioAnalysisData, frame: &mut [u8]);
    fn update_config(&mut self, config: Value);
    fn get_base_config(&self) -> BaseEffectConfig;
}

pub fn get_base_schema() -> Vec<EffectSetting> {
    vec![
        EffectSetting {
            id: "mirror".to_string(),
            name: "Mirror".to_string(),
            description: "Mirror the effect".to_string(),
            control: Control::Checkbox,
            default_value: DefaultValue::Bool(false),
        },
        EffectSetting {
            id: "flip".to_string(),
            name: "Flip".to_string(),
            description: "Flip the effect direction".to_string(),
            control: Control::Checkbox,
            default_value: DefaultValue::Bool(false),
        },
        EffectSetting {
            id: "blur".to_string(),
            name: "Blur".to_string(),
            description: "Amount to blur the effect".to_string(),
            control: Control::Slider { min: 0.0, max: 10.0, step: 0.1 },
            default_value: DefaultValue::Number(0.0),
        },
        EffectSetting {
            id: "background_color".to_string(),
            name: "Background Color".to_string(),
            description: "Color of Background".to_string(),
            control: Control::ColorPicker,
            default_value: DefaultValue::String("#000000".to_string()),
        },
    ]
}