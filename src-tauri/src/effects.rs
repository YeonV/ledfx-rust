// src-tauri/src/effects.rs

// --- The Effect Trait (The Contract) ---
pub trait Effect: Send {
    fn render_frame(&mut self, leds: u32) -> Vec<u8>;
}

// --- The Rainbow Effect ---
pub struct RainbowEffect {
    pub hue: f32,
}
impl Effect for RainbowEffect {
    fn render_frame(&mut self, leds: u32) -> Vec<u8> {
        self.hue = (self.hue + 1.0) % 360.0;
        let rgb = hsv_to_rgb(self.hue, 1.0, 1.0);
        let mut data = Vec::with_capacity((leds * 3) as usize);
        for _ in 0..leds {
            data.extend_from_slice(&rgb);
        }
        data
    }
}

// --- The Scan Effect ---
pub struct ScanEffect {
    pub position: u32,
    pub color: [u8; 3],
}
impl Effect for ScanEffect {
    fn render_frame(&mut self, leds: u32) -> Vec<u8> {
        self.position = (self.position + 1) % leds;
        let mut data = vec![0; (leds * 3) as usize];
        let start_index = (self.position * 3) as usize;
        if start_index + 3 <= data.len() {
            data[start_index..start_index + 3].copy_from_slice(&self.color);
        }
        data
    }
}

// --- The Scroll Effect ---
pub struct ScrollEffect {
    pub hue: f32,
}
impl Effect for ScrollEffect {
    fn render_frame(&mut self, leds: u32) -> Vec<u8> {
        self.hue = (self.hue + 0.5) % 360.0;
        let mut data = Vec::with_capacity((leds * 3) as usize);
        for i in 0..leds {
            let pixel_hue = (self.hue + (i as f32 * 10.0)) % 360.0;
            let rgb = hsv_to_rgb(pixel_hue, 1.0, 1.0);
            data.extend_from_slice(&rgb);
        }
        data
    }
}

// --- Helper Function ---
// This is kept here as it's directly related to creating effect colors.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r_prime, g_prime, b_prime) = if (0.0..60.0).contains(&h) { (c, x, 0.0) }
    else if (60.0..120.0).contains(&h) { (x, c, 0.0) }
    else if (120.0..180.0).contains(&h) { (0.0, c, x) }
    else if (180.0..240.0).contains(&h) { (0.0, x, c) }
    else if (240.0..300.0).contains(&h) { (x, 0.0, c) }
    else { (c, 0.0, x) };
    [((r_prime + m) * 255.0) as u8, ((g_prime + m) * 255.0) as u8, ((b_prime + m) * 255.0) as u8]
}