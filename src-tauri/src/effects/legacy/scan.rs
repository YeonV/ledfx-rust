use crate::effects::{BaseEffectConfig, Effect};
use crate::utils::colors::{self};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;
use crate::audio::AudioAnalysisData;
use super::blade_power::{EffectSetting, Control, DefaultValue};

// 1. Define the configuration for this specific effect
#[derive(Deserialize, Serialize, Type, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ScanLegacyConfig {
    pub speed: f32,
    pub width: f32,
    pub gradient: String,
    
    #[serde(flatten)]
    pub base: BaseEffectConfig,
}

// 2. Define the UI schema for the configuration
pub fn get_schema() -> Vec<EffectSetting> {
    vec![
        // Base settings are duplicated for now. We can refactor this later.
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
        // Effect-specific settings
        EffectSetting {
            id: "speed".to_string(),
            name: "Speed".to_string(),
            description: "Speed of the scanner".to_string(),
            control: Control::Slider { min: 1.0, max: 10.0, step: 0.1 },
            default_value: DefaultValue::Number(1.0),
        },
        EffectSetting {
            id: "width".to_string(),
            name: "Width".to_string(),
            description: "Width of the scanner".to_string(),
            control: Control::Slider { min: 1.0, max: 50.0, step: 1.0 },
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
    ]
}

// 3. Define the effect's runtime state and implementation
pub struct ScanLegacy {
    pub config: ScanLegacyConfig,
    gradient_palette: Vec<[u8; 3]>,
    position: f32,
}

impl ScanLegacy {
    pub fn new(config: ScanLegacyConfig) -> Self {
        Self {
            config,
            gradient_palette: Vec::new(),
            position: 0.0,
        }
    }

    fn rebuild_palette(&mut self) {
        // The width is in pixels, but let's treat it as an integer for the palette size
        let palette_size = self.config.width.ceil() as usize;
        self.gradient_palette = colors::parse_gradient(&self.config.gradient, palette_size);
    }
}

impl Effect for ScanLegacy {
    fn render(&mut self, _audio_data: &AudioAnalysisData, frame: &mut [u8]) {
        let pixel_count = frame.len() / 3;
        if pixel_count == 0 { return; }

        let width = self.config.width.ceil() as usize;
        if self.gradient_palette.len() != width {
            self.rebuild_palette();
        }
        if self.gradient_palette.is_empty() { return; } // Don't render if palette is invalid

        // Update position based on speed
        self.position = (self.position + self.config.speed) % (pixel_count as f32);
        
        // Clear the frame to black
        frame.fill(0);

        // Draw the gradient at the current position
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
            self.gradient_palette.clear(); // Invalidate palette to force rebuild
        } else {
            eprintln!("Failed to deserialize settings for ScanLegacy");
        }
    }
    
    fn get_base_config(&self) -> BaseEffectConfig {
        self.config.base.clone()
    }
}