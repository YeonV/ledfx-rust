// src-tauri/src/utils/colors.rs

// This is a simplified parser. A real one would handle multiple color stops.
pub fn parse_gradient(_gradient_str: &str, num_leds: usize) -> Vec<[u8; 3]> {
    let mut palette = Vec::with_capacity(num_leds);
    for i in 0..num_leds {
        let ratio = i as f32 / (num_leds.saturating_sub(1)) as f32;
        let r = (255.0 * (1.0 - ratio)) as u8;
        let b = (255.0 * ratio) as u8;
        palette.push([r, 0, b]);
    }
    palette
}

pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r_prime, g_prime, b_prime) = if (0.0..60.0).contains(&h) { (c, x, 0.0) }
    else if (60.0..120.0).contains(&h) { (x, c, 0.0) }
    else if (120.0..180.0).contains(&h) { (0.0, c, x) }
    else if (180.0..240.0).contains(&h) { (0.0, x, c) }
    else if (240.0..300.0).contains(&h) { (x, 0.0, c) }
    else { (c, 0.0, x) };
    [
        ((r_prime + m) * 255.0) as u8,
        ((g_prime + m) * 255.0) as u8,
        ((b_prime + m) * 255.0) as u8,
    ]
}