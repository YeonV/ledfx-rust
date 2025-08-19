// src-tauri/src/audio/android.rs

use jni::objects::{JClass};
use jni::JNIEnv;
use jni::sys::{jbyteArray, jint};
use once_cell::sync::Lazy;
use serde::Serialize;
use specta::Type;
use std::sync::{mpsc, Arc, Mutex};
use super::{AudioAnalysisData, AudioDevice, AudioCommand};

static SHARED_AUDIO_DATA: Lazy<Arc<Mutex<AudioAnalysisData>>> = Lazy::new(Default::default);

// --- THE FIX: The function now has the same signature as the desktop version ---
pub fn start_audio_capture(
    _command_rx: mpsc::Receiver<AudioCommand>, // This is ignored on Android
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
    fft: jbyteArray,
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
    Ok(vec![AudioDevice { name: "System Audio (Native)".to_string() }])
}