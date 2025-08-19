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

// --- THE FIX: Make the sub-modules fully public ---
#[cfg(target_os = "android")]
pub mod android;
#[cfg(not(target_os = "android"))]
pub mod desktop;