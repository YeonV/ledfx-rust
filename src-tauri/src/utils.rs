// src-tauri/src/utils.rs

pub mod colors; // <-- NEW: Declare the colors module
pub mod ddp; // <-- NEW: Declare the ddp module

// --- THE FIX: Re-export the functions to maintain the flat API ---
pub use colors::hsv_to_rgb;
pub use colors::parse_gradient;
pub use ddp::send_ddp_packet;
