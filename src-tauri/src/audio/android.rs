// src-tauri/src/audio/android.rs

use jni::objects::{JClass, JByteArray};
use jni::JNIEnv;
use jni::sys::{jint};
use once_cell::sync::Lazy;
use std::sync::{mpsc, Arc, Mutex};
use tauri::State;
use super::{AudioAnalysisData, AudioDevice};
use cpal::traits::{DeviceTrait, HostTrait};

static SHARED_AUDIO_DATA: Lazy<Arc<Mutex<AudioAnalysisData>>> = Lazy::new(Default::default);

pub enum AudioCommand {}

pub fn start_audio_capture(
    _command_rx: mpsc::Receiver<AudioCommand>,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
) {
    let mut global_data = SHARED_AUDIO_DATA.lock().unwrap();
    *global_data = audio_data.lock().unwrap().clone();
    println!("ANDROID AUDIO: Native capture thread started (simulated).");
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_blade_ledfxrust_AudioVisualizer_onFftDataCapture(
    env: JNIEnv,
    _class: JClass,
    fft: JByteArray,
    _sampling_rate: jint,
) {
    let fft_bytes = env.convert_byte_array(fft).unwrap();
    let magnitudes: Vec<f32> = fft_bytes.iter().map(|&b| b as f32 / 128.0).collect();
    const GAIN: f32 = 30.0;
    let sum: f32 = magnitudes.iter().sum();
    let volume = (sum / magnitudes.len() as f32 * GAIN).min(1.0);
    let mut analysis_data = SHARED_AUDIO_DATA.lock().unwrap();
    analysis_data.volume = volume;
}

#[tauri::command]
#[specta::specta]
pub fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let mut device_list: Vec<AudioDevice> = Vec::new();
    device_list.push(AudioDevice { name: "System Audio (Native Visualizer)".to_string() });

    let host = cpal::default_host();
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                device_list.push(AudioDevice { name });
            }
        }
    }
    Ok(device_list)
}

#[tauri::command]
#[specta::specta]
pub fn set_audio_device(
    _device_name: String,
    _command_tx: State<mpsc::Sender<AudioCommand>>,
) -> Result<(), String> {
    println!("set_audio_device on Android will select between Native and CPAL in the future.");
    Ok(())
}