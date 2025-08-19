// src-tauri/src/audio/mod.rs

use std::sync::{Arc, Mutex};
use serde::Serialize;
use specta::Type;

#[derive(Serialize, Clone, Type)]
pub struct AudioDevice {
    pub name: String,
}

#[derive(Default, Clone)]
pub struct AudioAnalysisData {
    pub volume: f32,
}

#[derive(Default)]
pub struct SharedAudioData(pub Arc<Mutex<AudioAnalysisData>>);

// --- THE FIX: Make sub-modules visible within the crate ---
#[cfg(target_os = "android")]
pub(crate) mod android;
#[cfg(not(target_os = "android"))]
pub(crate) mod desktop;

// --- THE FIX: Create a consistent public API using pub use ---

#[cfg(target_os = "android")]
pub use android::{get_audio_devices, start_audio_capture, set_audio_device, AudioCommand};
#[cfg(not(target_os = "android"))]
pub use desktop::{get_audio_devices, start_audio_capture, set_audio_device, AudioCommand};