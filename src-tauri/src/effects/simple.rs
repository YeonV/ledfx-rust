use crate::audio::AudioAnalysisData;
use crate::effects::{BaseEffectConfig, Effect, legacy::blade_power::{EffectSetting, Control, DefaultValue}};
use crate::utils::colors::hsv_to_rgb;
use serde_json::Value;

// --- START: NEW SHARED SCHEMA FOR SIMPLE EFFECTS ---
// Since simple effects only have base settings, we can share this schema.
pub fn get_simple_schema() -> Vec<EffectSetting> {
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
            default_value: DefaultValue::Number(0.0),
        },
        EffectSetting {
            id: "background_color".to_string(),
            name: "Background Color".to_string(),
            description: "Color of Background".to_string(),
            control: Control::ColorPicker,
            default_value: DefaultValue::String("#000000".to_string()),
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
// --- END: NEW SHARED SCHEMA ---


pub struct RainbowEffect {
    // It no longer needs state
    config: BaseEffectConfig,
}

impl RainbowEffect {
    pub fn new() -> Self {
        Self {
            config: BaseEffectConfig {
                mirror: false,
                flip: false,
                blur: 0.0,
                background_color: "#000000".to_string(),
            }
        }
    }
}

impl Effect for RainbowEffect {
    fn render(&mut self, _audio_data: &AudioAnalysisData, frame: &mut [u8]) {
        let pixel_count = frame.len() / 3;
        for i in 0..pixel_count {
            let hue = (i as f32 * 360.0) / pixel_count as f32;
            let rgb = hsv_to_rgb(hue, 1.0, 1.0);
            frame[i * 3] = rgb[0];
            frame[i * 3 + 1] = rgb[1];
            frame[i * 3 + 2] = rgb[2];
        }
    }

    fn update_config(&mut self, config: Value) {
        if let Ok(new_config) = serde_json::from_value(config) {
            self.config = new_config;
        }
    }

    fn get_base_config(&self) -> BaseEffectConfig {
        self.config.clone()
    }
}

// ScanEffect is removed from here

// --- FadeEffect is the new name for the scrolling rainbow ---
pub struct FadeEffect {
    pub hue: f32,
    config: BaseEffectConfig,
}

impl FadeEffect {
    pub fn new() -> Self {
        Self {
            hue: 0.0,
            config: BaseEffectConfig {
                mirror: false,
                flip: false,
                blur: 0.0,
                background_color: "#000000".to_string(),
            }
        }
    }
}

impl Effect for FadeEffect {
    fn render(&mut self, _audio_data: &AudioAnalysisData, frame: &mut [u8]) {
        let pixel_count = frame.len() / 3;
        self.hue = (self.hue + 0.5) % 360.0;
        
        for i in 0..pixel_count {
            let pixel_hue = (self.hue + (i as f32 * 10.0)) % 360.0;
            let rgb = hsv_to_rgb(pixel_hue, 1.0, 1.0);
            let start_index = i * 3;
            frame[start_index] = rgb[0];
            frame[start_index + 1] = rgb[1];
            frame[start_index + 2] = rgb[2];
        }
    }

    fn update_config(&mut self, config: Value) {
        if let Ok(new_config) = serde_json::from_value(config) {
            self.config = new_config;
        }
    }

    fn get_base_config(&self) -> BaseEffectConfig {
        self.config.clone()
    }
}