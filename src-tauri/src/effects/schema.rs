use serde::{Deserialize, Serialize};
use specta::Type;

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