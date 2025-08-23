use crate::audio::AudioAnalysisData;
use serde_json::Value;

pub mod blade;
pub mod legacy;
pub mod simple;

// This is our single source of truth for what an effect is.
pub trait Effect: Send + Sync {
    /// Renders a single frame of the effect into the provided buffer.
    fn render(&mut self, audio_data: &AudioAnalysisData, frame: &mut [u8]);

    /// Updates the effect's configuration from a JSON value.
    fn update_config(&mut self, config: Value);
}