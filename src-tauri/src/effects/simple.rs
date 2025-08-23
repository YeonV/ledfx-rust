use crate::audio::AudioAnalysisData;
use crate::effects::Effect;
use crate::utils::colors::hsv_to_rgb;
use serde_json::Value;

pub struct RainbowEffect {
    pub hue: f32,
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

    fn update_config(&mut self, _config: Value) {}
}

pub struct ScanEffect {
    pub position: u32,
    pub color: [u8; 3],
}

impl Effect for ScanEffect {
    fn render(&mut self, _audio_data: &AudioAnalysisData, frame: &mut [u8]) {
        let pixel_count = (frame.len() / 3) as u32;
        if pixel_count == 0 { return; }
        
        self.position = (self.position + 1) % pixel_count;
        frame.fill(0); // Clear the frame
        
        let start_index = (self.position * 3) as usize;
        if start_index + 2 < frame.len() {
            frame[start_index] = self.color[0];
            frame[start_index + 1] = self.color[1];
            frame[start_index + 2] = self.color[2];
        }
    }

    fn update_config(&mut self, _config: Value) {}
}

pub struct ScrollEffect {
    pub hue: f32,
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

    fn update_config(&mut self, _config: Value) {}
}