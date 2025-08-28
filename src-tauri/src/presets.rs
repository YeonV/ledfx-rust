use crate::engine::EffectConfig;
// use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type EffectPresetMap = HashMap<String, HashMap<String, EffectConfig>>;
