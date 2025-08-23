use regex::Regex;

// Main public function. Takes a CSS string and generates a palette of a given size.
pub fn parse_gradient(gradient_str: &str, size: usize) -> Vec<[u8; 3]> {
    // If it's a simple color (hex or rgb), create a solid palette.
    if !gradient_str.starts_with("linear-gradient") {
        let color = parse_single_color(gradient_str).unwrap_or([0, 0, 0]);
        return vec![color; size];
    }

    // It's a gradient, so parse the color stops.
    let stops = parse_color_stops(gradient_str);
    if stops.is_empty() {
        return vec![[0, 0, 0]; size]; // Return black if parsing fails
    }

    // Generate the final palette by interpolating between the stops.
    generate_palette_from_stops(&stops, size)
}

// Helper to parse a single color token (hex, short-hex, or rgb).
fn parse_single_color(color_str: &str) -> Option<[u8; 3]> {
    let s = color_str.trim();
    if s.starts_with('#') {
        let hex = s.strip_prefix('#').unwrap();
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some([r, g, b])
        } else if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
            Some([r, g, b])
        } else {
            None
        }
    } else if s.starts_with("rgb") {
        let inner = s.strip_prefix("rgb(")?.strip_suffix(")")?;
        let mut parts = inner.split(',').map(|p| p.trim().parse::<u8>());
        let r = parts.next()?.ok()?;
        let g = parts.next()?.ok()?;
        let b = parts.next()?.ok()?;
        Some([r, g, b])
    } else {
        None
    }
}

// Uses regex to find all color stops in a `linear-gradient` string.
fn parse_color_stops(gradient_str: &str) -> Vec<(f32, [u8; 3])> {
    // This regex captures the color part (hex or rgb) and its percentage.
    let re = Regex::new(r"(#[0-9a-fA-F]{3,6}|rgb\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*\))\s+([\d.]+)%").unwrap();
    let mut stops: Vec<(f32, [u8; 3])> = re.captures_iter(gradient_str).filter_map(|cap| {
        let color_str = cap.get(1).map_or("", |m| m.as_str());
        let percent_str = cap.get(2).map_or("", |m| m.as_str());
        
        let color = parse_single_color(color_str)?;
        let percent = percent_str.parse::<f32>().ok()? / 100.0;
        
        Some((percent, color))
    }).collect();

    // Ensure stops are sorted by percentage.
    stops.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    stops
}

// Generates the final palette by interpolating between the parsed color stops.
fn generate_palette_from_stops(stops: &[(f32, [u8; 3])], size: usize) -> Vec<[u8; 3]> {
    if stops.is_empty() || size == 0 {
        return Vec::new();
    }
    if stops.len() == 1 {
        return vec![stops[0].1; size];
    }

    let mut palette = Vec::with_capacity(size);

    for i in 0..size {
        let pos = i as f32 / (size - 1) as f32;
        
        // Find the two stops the current position is between.
        let end_stop_idx = stops.iter().position(|s| s.0 >= pos).unwrap_or(stops.len() - 1);
        let start_stop_idx = if end_stop_idx > 0 { end_stop_idx - 1 } else { 0 };

        let start_stop = &stops[start_stop_idx];
        let end_stop = &stops[end_stop_idx];

        // Calculate how far we are between the start and end stops.
        let t = if (end_stop.0 - start_stop.0).abs() < 1e-6 {
            0.0 // Avoid division by zero if stops are at the same position
        } else {
            (pos - start_stop.0) / (end_stop.0 - start_stop.0)
        };

        // Linear interpolation for each color channel.
        let r = start_stop.1[0] as f32 * (1.0 - t) + end_stop.1[0] as f32 * t;
        let g = start_stop.1[1] as f32 * (1.0 - t) + end_stop.1[1] as f32 * t;
        let b = start_stop.1[2] as f32 * (1.0 - t) + end_stop.1[2] as f32 * t;

        palette.push([r as u8, g as u8, b as u8]);
    }
    palette
}

pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r_prime, g_prime, b_prime) = if (0.0..60.0).contains(&h) {
        (c, x, 0.0)
    } else if (60.0..120.0).contains(&h) {
        (x, c, 0.0)
    } else if (120.0..180.0).contains(&h) {
        (0.0, c, x)
    } else if (180.0..240.0).contains(&h) {
        (0.0, x, c)
    } else if (240.0..300.0).contains(&h) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    [
        ((r_prime + m) * 255.0) as u8,
        ((g_prime + m) * 255.0) as u8,
        ((b_prime + m) * 255.0) as u8,
    ]
}
