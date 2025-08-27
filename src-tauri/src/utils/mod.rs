// src-tauri/src/utils/mod.rs

pub mod colors;
pub mod ddp;
pub mod dsp;

// Re-export the most used functions for convenience
pub use colors::hsv_to_rgb;
pub use colors::parse_gradient;
pub use ddp::send_ddp_packet;
