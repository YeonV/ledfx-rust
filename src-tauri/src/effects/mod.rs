use crate::audio::AudioAnalysisData;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use crate::effects::schema::{Control, DefaultValue, EffectSetting};

pub mod blade_power;
pub mod scan;

pub mod schema;

#[derive(Deserialize, Serialize, Type, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct BaseEffectConfig { /* ... fields ... */ }

pub trait Effect: Send + Sync { /* ... methods ... */ }

pub fn get_base_schema() -> Vec<EffectSetting> { /* ... body ... */ }
