// src-tauri/src/audio/mod.rs

// Make the sub-modules public so lib.rs can see them.
pub mod devices;
pub mod capture;

// Re-export the necessary items to the rest of the application.
pub use devices::{AudioDevice, get_audio_devices, set_audio_device};
pub use capture::{AudioAnalysisData, SharedAudioData, AudioCommand, run_audio_capture};