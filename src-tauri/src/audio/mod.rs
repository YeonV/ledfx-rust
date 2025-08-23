use serde::Serialize;
use specta::Type; // FIX: Corrected import path for `Type`
use std::sync::{mpsc, Arc, Mutex};
use tauri::State;

#[derive(Serialize, Clone, Type)]
pub struct AudioDevice {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Type)]
pub struct AudioAnalysisData {
    pub melbanks: Vec<f32>,
}

impl Default for AudioAnalysisData {
    fn default() -> Self {
        Self::new(128)
    }
}

impl AudioAnalysisData {
    const BASS_LOW: usize = 0;
    const BASS_HIGH: usize = 15;
    const MIDS_LOW: usize = 16;
    const MIDS_HIGH: usize = 63;
    const HIGHS_LOW: usize = 64;
    const HIGHS_HIGH: usize = 127;

    pub fn new(num_bands: usize) -> Self {
        Self {
            melbanks: vec![0.0; num_bands],
        }
    }
    
    pub fn lows_power(&self) -> f32 {
        let high = Self::BASS_HIGH.min(self.melbanks.len().saturating_sub(1));
        let low = Self::BASS_LOW.min(high);
        if low >= high { return 0.0; }
        let slice = &self.melbanks[low..=high];
        slice.iter().sum::<f32>() / slice.len() as f32
    }

    pub fn mids_power(&self) -> f32 {
        let high = Self::MIDS_HIGH.min(self.melbanks.len().saturating_sub(1));
        let low = Self::MIDS_LOW.min(high);
        if low >= high { return 0.0; }
        let slice = &self.melbanks[low..=high];
        slice.iter().sum::<f32>() / slice.len() as f32
    }

    pub fn highs_power(&self) -> f32 {
        let high = Self::HIGHS_HIGH.min(self.melbanks.len().saturating_sub(1));
        let low = Self::HIGHS_LOW.min(high);
        if low >= high { return 0.0; }
        let slice = &self.melbanks[low..=high];
        slice.iter().sum::<f32>() / slice.len() as f32
    }
}

#[derive(Default)]
pub struct SharedAudioData(pub Arc<Mutex<AudioAnalysisData>>);

#[cfg(target_os = "android")]
pub mod android;
#[cfg(not(target_os = "android"))]
pub mod desktop;
pub mod devices;

pub enum AudioCommand {
    ChangeDevice(String),
}

pub fn start_audio_capture(
    command_rx: mpsc::Receiver<AudioCommand>,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
) {
    #[cfg(not(target_os = "android"))]
    desktop::run_desktop_capture(command_rx, audio_data);
    #[cfg(target_os = "android")]
    android::run_android_capture(command_rx, audio_data);
}

// --- TAURI COMMANDS ---

#[tauri::command]
#[specta::specta]
pub fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    #[cfg(not(target_os = "android"))]
    return devices::get_desktop_devices_impl();
    #[cfg(target_os = "android")]
    return Ok(android::get_android_devices());
}

#[tauri::command]
#[specta::specta]
pub fn set_audio_device(
    device_name: String,
    command_tx: State<mpsc::Sender<AudioCommand>>,
) -> Result<(), String> {
    #[cfg(not(target_os = "android"))]
    return devices::set_desktop_device_impl(device_name, command_tx);
    #[cfg(target_os = "android")]
    return android::set_android_device(device_name, command_tx);
}

#[tauri::command]
#[specta::specta]
pub fn get_audio_analysis(
    audio_data: State<SharedAudioData>,
) -> Result<AudioAnalysisData, String> {
    let data = audio_data.0.lock().map_err(|e| e.to_string())?;
    Ok(data.clone())
}