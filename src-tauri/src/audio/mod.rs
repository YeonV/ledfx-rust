use crate::utils::dsp::{BladePlusParams, FilterbankType};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::{mpsc, Arc, Mutex};
use tauri::State;
mod shared_processing;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DspSettings {
    // Critical Settings (require audio stream restart)
    pub fft_size: u32,
    pub num_bands: u32,
    pub min_freq: f32,
    pub max_freq: f32,
    pub filterbank_type: FilterbankType,
    pub sample_rate: Option<u32>,
    pub blade_plus_params: Option<BladePlusParams>,

    // Live Settings (can be changed on the fly)
    pub smoothing_factor: f32,
    pub agc_attack: f32,
    pub agc_decay: f32,
    pub audio_delay_ms: u32,
}

impl Default for DspSettings {
    fn default() -> Self {
        Self {
            fft_size: 4096,
            num_bands: 24,
            min_freq: 20.0,
            max_freq: 15000.0,
            filterbank_type: FilterbankType::Blade,
            sample_rate: Some(30000),
            blade_plus_params: Some(BladePlusParams {
                log_base: 12.0,
                multiplier: 3700.0,
                divisor: 230.0,
            }),
            smoothing_factor: 0.4,
            agc_attack: 0.01,
            agc_decay: 0.1,
            audio_delay_ms: 0,
        }
    }
}

#[derive(Default, Clone)]
pub struct SharedDspSettings(pub Arc<Mutex<DspSettings>>);

#[derive(Serialize, Clone, Type)]
pub struct AudioDevice {
    pub name: String,
}

#[derive(Serialize, Type, Clone)]
pub struct AudioDevicesInfo {
    pub devices: Vec<AudioDevice>,
    pub default_device_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Type, Default)]
pub struct AudioAnalysisData {
    pub melbanks: Vec<f32>,
}

impl AudioAnalysisData {
    pub fn new(num_bands: usize) -> Self {
        Self {
            melbanks: vec![0.0; num_bands],
        }
    }
}

const BASS_LOW: usize = 0;
const BASS_HIGH: usize = 15;
const MIDS_LOW: usize = 16;
const MIDS_HIGH: usize = 63;
const HIGHS_LOW: usize = 64;
const HIGHS_HIGH: usize = 127;

pub fn lows_power(melbanks: &[f32]) -> f32 {
    let high = BASS_HIGH.min(melbanks.len().saturating_sub(1));
    let low = BASS_LOW.min(high);
    if low >= high {
        return 0.0;
    }
    let slice = &melbanks[low..=high];
    slice.iter().sum::<f32>() / slice.len() as f32
}

pub fn mids_power(melbanks: &[f32]) -> f32 {
    let high = MIDS_HIGH.min(melbanks.len().saturating_sub(1));
    let low = MIDS_LOW.min(high);
    if low >= high {
        return 0.0;
    }
    let slice = &melbanks[low..=high];
    slice.iter().sum::<f32>() / slice.len() as f32
}

pub fn highs_power(melbanks: &[f32]) -> f32 {
    let high = HIGHS_HIGH.min(melbanks.len().saturating_sub(1));
    let low = HIGHS_LOW.min(high);
    if low >= high {
        return 0.0;
    }
    let slice = &melbanks[low..=high];
    slice.iter().sum::<f32>() / slice.len() as f32
}

#[derive(Default, Clone)]
pub struct SharedAudioData(pub Arc<Mutex<AudioAnalysisData>>);

#[cfg(target_os = "android")]
pub mod android;
#[cfg(not(target_os = "android"))]
pub mod desktop;

pub enum AudioCommand {
    ChangeDevice(String),
    UpdateSettings(DspSettings),
    RestartStream,
}

pub fn start_audio_capture(
    command_rx: mpsc::Receiver<AudioCommand>,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
    dsp_settings: Arc<Mutex<DspSettings>>,
) {
    #[cfg(not(target_os = "android"))]
    desktop::run_desktop_capture(command_rx, audio_data, dsp_settings);
    #[cfg(target_os = "android")]
    android::run_android_capture(command_rx, audio_data, dsp_settings);
}

#[tauri::command]
#[specta::specta]
pub fn get_dsp_settings(settings: State<SharedDspSettings>) -> Result<DspSettings, String> {
    let data = settings.0.lock().map_err(|e| e.to_string())?;
    Ok(data.clone())
}

#[tauri::command]
#[specta::specta]
pub fn get_audio_devices() -> Result<AudioDevicesInfo, String> {
    #[cfg(not(target_os = "android"))]
    return desktop::get_desktop_devices_impl();
    #[cfg(target_os = "android")]
    return Ok(AudioDevicesInfo {
        devices: vec![],
        default_device_name: None,
    });
}

#[tauri::command]
#[specta::specta]
pub fn set_audio_device(
    device_name: String,
    command_tx: State<mpsc::Sender<AudioCommand>>,
) -> Result<(), String> {
    #[cfg(not(target_os = "android"))]
    return crate::audio::desktop::set_desktop_device_impl(device_name, command_tx);
    #[cfg(target_os = "android")]
    return crate::audio::android::set_android_device(device_name, command_tx);
}

#[tauri::command]
#[specta::specta]
pub fn get_audio_analysis(audio_data: State<SharedAudioData>) -> Result<AudioAnalysisData, String> {
    let data = audio_data.0.lock().map_err(|e| e.to_string())?;
    Ok(data.clone())
}
