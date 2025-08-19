// src-tauri/src/audio/mod.rs

use std::sync::{mpsc, Arc, Mutex};
use serde::Serialize;
use specta::Type;
use tauri::State;

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

#[cfg(target_os = "android")]
pub(crate) mod android;
#[cfg(not(target_os = "android"))]
pub(crate) mod desktop;

// This is the single, stable AudioCommand enum for the whole application.
pub enum AudioCommand {
    ChangeDevice(String),
}

// This is the single, stable start_audio_capture function.
pub fn start_audio_capture(
    command_rx: mpsc::Receiver<AudioCommand>,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
) {
    #[cfg(not(target_os = "android"))]
    desktop::run_desktop_capture(command_rx, audio_data);
    #[cfg(target_os = "android")]
    // --- THE FIX: Call the correct function name ---
    android::start_audio_capture(command_rx, audio_data);
}

// This is the single, stable get_audio_devices command.
#[tauri::command]
#[specta::specta]
pub fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    #[cfg(not(target_os = "android"))]
    return desktop::get_desktop_devices();
    #[cfg(target_os = "android")]
    return android::get_android_devices();
}

// This is the single, stable set_audio_device command.
#[tauri::command]
#[specta::specta]
pub fn set_audio_device(
    device_name: String,
    command_tx: State<mpsc::Sender<AudioCommand>>,
) -> Result<(), String> {
    #[cfg(not(target_os = "android"))]
    return desktop::set_desktop_device(device_name, command_tx);
    #[cfg(target_os = "android")]
    return android::set_android_device(device_name, command_tx);
}