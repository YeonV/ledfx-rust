use crate::audio::AudioAnalysisData;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;

pub mod blade;
pub mod legacy;
pub mod simple;

#[derive(Deserialize, Serialize, Type, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct BaseEffectConfig {
    pub mirror: bool,
    pub flip: bool,
    pub blur: f32,
    pub background_color: String,
}

pub trait Effect: Send + Sync {
    /// Renders the core visual logic of the effect. Post-processing will be handled by the engine.
    fn render(&mut self, audio_data: &AudioAnalysisData, frame: &mut [u8]);

    /// Updates the effect's specific configuration.
    fn update_config(&mut self, config: Value);
    
    /// Provides the engine with access to the common post-processing settings.
    fn get_base_config(&self) -> BaseEffectConfig;
}