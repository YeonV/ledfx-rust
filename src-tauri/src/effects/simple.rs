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
    pub hue: f32,
    config: BaseEffectConfig,
}

impl RainbowEffect {
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

impl Effect for RainbowEffect {
    fn render(&mut self, _audio_data: &AudioAnalysisData, frame: &mut [u8]) {
        self.hue = (self.hue + 1.0) % 360.0;
        let rgb = hsv_to_rgb(self.hue, 1.0, 1.0);
        for pixel in frame.chunks_mut(3) {
            pixel[0] = rgb[0];
            pixel[1] = rgb[1];
            pixel[2] = rgb[2];
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

pub struct ScanEffect {
    pub position: u32,
    pub color: [u8; 3],
    config: BaseEffectConfig,
}

impl ScanEffect {
    pub fn new() -> Self {
        Self {
            position: 0,
            color: [255, 0, 0],
            config: BaseEffectConfig {
                mirror: false,
                flip: false,
                blur: 0.0,
                background_color: "#000000".to_string(),
            }
        }
    }
}

impl Effect for ScanEffect {
    fn render(&mut self, _audio_data: &AudioAnalysisData, frame: &mut [u8]) {
        let pixel_count = (frame.len() / 3) as u32;
        if pixel_count == 0 { return; }
        
        self.position = (self.position + 1) % pixel_count;
        frame.fill(0);
        
        let start_index = (self.position * 3) as usize;
        if start_index + 2 < frame.len() {
            frame[start_index] = self.color[0];
            frame[start_index + 1] = self.color[1];
            frame[start_index + 2] = self.color[2];
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

pub struct ScrollEffect {
    pub hue: f32,
    config: BaseEffectConfig,
}

impl ScrollEffect {
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

impl Effect for ScrollEffect {
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