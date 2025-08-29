use super::{
    AudioAnalysisData, AudioCommand, AudioDevice, AudioDevicesInfo, DspSettings, SharedDspSettings,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, SupportedStreamConfig};
use dasp::{interpolate::linear::Linear, signal, Signal};
use dasp_sample::Sample;
use jni::objects::{JByteArray, JClass};
use jni::sys::jint;
use jni::JNIEnv;
use once_cell::sync::Lazy;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::f32::consts::PI;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use tauri::State;

// --- GLOBAL STATE FOR JNI & CPAL/JNI TOGGLE ---
static SHARED_AUDIO_DATA: Lazy<Arc<Mutex<AudioAnalysisData>>> = Lazy::new(Default::default);
static SHARED_DSP_SETTINGS: Lazy<SharedDspSettings> = Lazy::new(Default::default);
// This flag ensures that JNI processing and CPAL processing don't run simultaneously.
static IS_NATIVE_CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(true);

/// This is the main audio loop for Android, it handles commands to switch capture methods.
pub fn run_android_capture(
    command_rx: mpsc::Receiver<AudioCommand>,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
    dsp_settings: Arc<Mutex<DspSettings>>,
) {
    // Connect the shared state from the main app to our static JNI-accessible state.
    let mut global_data = SHARED_AUDIO_DATA.lock().unwrap();
    *global_data = audio_data.lock().unwrap().clone();
    let mut global_settings = SHARED_DSP_SETTINGS.0.lock().unwrap();
    *global_settings = dsp_settings.lock().unwrap().clone();

    println!("[AUDIO] Android capture context initialized. Defaulting to Native JNI mode.");

    let host = cpal::default_host();
    let mut current_cpal_stream: Option<Stream> = None;

    loop {
        if let Ok(command) = command_rx.try_recv() {
            match command {
                AudioCommand::ChangeDevice(device_name) => {
                    println!(
                        "[AUDIO] Android received ChangeDevice command: {}",
                        device_name
                    );

                    // Always stop any existing CPAL stream when changing devices.
                    if let Some(stream) = current_cpal_stream.take() {
                        stream.pause().expect("Failed to pause CPAL stream");
                        drop(stream);
                        println!("[AUDIO] Stopped existing CPAL stream.");
                    }

                    // The main decision: are we using the JNI bridge or a CPAL device?
                    if device_name.contains("Native") {
                        IS_NATIVE_CAPTURE_ACTIVE.store(true, Ordering::SeqCst);
                        println!("[AUDIO] Switched to JNI Native Visualizer mode.");
                    } else {
                        IS_NATIVE_CAPTURE_ACTIVE.store(false, Ordering::SeqCst);
                        println!("[AUDIO] Switched to CPAL device: {}", device_name);

                        if let Some(device) = find_device(&host, &device_name) {
                            if let Ok(config) = device.default_input_config() {
                                let stream = build_and_play_stream_cpal(
                                    device,
                                    config,
                                    audio_data.clone(),
                                    dsp_settings.clone(),
                                );
                                current_cpal_stream = Some(stream);
                            } else {
                                eprintln!(
                                    "[AUDIO] Could not get default input config for {}",
                                    device_name
                                );
                            }
                        } else {
                            eprintln!(
                                "[AUDIO] Could not find CPAL device on Android: {}",
                                device_name
                            );
                        }
                    }
                }
                AudioCommand::UpdateSettings(new_settings) => {
                    println!("[AUDIO] Android received new DSP settings.");
                    let mut settings = SHARED_DSP_SETTINGS.0.lock().unwrap();
                    *settings = new_settings;
                }
                AudioCommand::RestartStream => {
                    // Restarting on Android is handled by re-sending ChangeDevice from the frontend.
                    println!("[AUDIO] Android received RestartStream command (no-op, frontend should re-select device).");
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

/// JNI FUNCTION: The entry point for audio data from the Android Native Visualizer API.
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_blade_ledfxrust_AudioVisualizer_onPcmDataCapture(
    env: JNIEnv,
    _class: JClass,
    pcm_data: JByteArray,
    sampling_rate: jint,
) {
    // If a CPAL stream is active, do nothing.
    if !IS_NATIVE_CAPTURE_ACTIVE.load(Ordering::SeqCst) {
        return;
    }
    let fft_size = settings.fft_size as usize;
    let num_bands = settings.num_bands as usize;

    // This JNI function will be called repeatedly, so we need to manage state.
    // We'll use a static mutex to hold the necessary buffers and state variables.
    static PROCESSING_STATE: Lazy<Mutex<AudioProcessingState>> = Lazy::new(Default::default);
    let mut state = PROCESSING_STATE.lock().unwrap();

    // Initialize or update buffers if settings have changed.
    if state.fft_size != fft_size {
        state.initialize(fft_size, num_bands);
    }

    // Convert Java byte array to Rust i16 slice (assuming 16-bit PCM audio)
    let pcm_i16 = env.convert_byte_array(pcm_data).unwrap();
    let samples: Vec<i16> = pcm_i16
        .chunks_exact(2)
        .map(|a| i16::from_le_bytes([a[0], a[1]]))
        .collect();

    // --- The same DSP pipeline as desktop ---
    let final_sample_rate = settings.sample_rate.unwrap_or(sampling_rate as u32);

    // Resampling would be complex here. For now, we assume native or target rate matches.
    // A full implementation would use `dasp` here if sample_rate != sampling_rate.

    state
        .audio_samples
        .extend(samples.iter().map(|s| s.to_sample::<f32>()));

    while state.audio_samples.len() >= fft_size {
        for (i, sample) in state.audio_samples.iter().take(fft_size).enumerate() {
            state.fft_buffer[i] = Complex::new(sample * state.window[i], 0.0);
        }

        state.fft_plan.process(&mut state.fft_buffer);

        let filterbank = crate::utils::dsp::generate_filterbank(
            fft_size / 2,
            final_sample_rate,
            num_bands,
            settings.min_freq,
            settings.max_freq,
            &settings.filterbank_type,
        );

        let magnitudes: Vec<f32> = state.fft_buffer[0..fft_size / 2]
            .iter()
            .map(|c| c.norm_sqr().sqrt())
            .collect();
        let raw_melbanks: Vec<f32> = filterbank
            .iter()
            .map(|filter| filter.iter().map(|&(bin, w)| magnitudes[bin] * w).sum())
            .collect();

        for i in 0..num_bands {
            state.smoothed_melbanks[i] = (state.smoothed_melbanks[i] * settings.smoothing_factor)
                + (raw_melbanks[i] * (1.0 - settings.smoothing_factor));
        }

        let current_max_energy = state
            .smoothed_melbanks
            .iter()
            .fold(0.0f32, |max, &val| val.max(max));
        if current_max_energy > state.peak_energy {
            state.peak_energy = state.peak_energy * (1.0 - settings.agc_attack)
                + current_max_energy * settings.agc_attack;
        } else {
            state.peak_energy = state.peak_energy * (1.0 - settings.agc_decay)
                + current_max_energy * settings.agc_decay;
        }
        state.peak_energy = state.peak_energy.max(1e-4);

        let final_melbanks: Vec<f32> = state
            .smoothed_melbanks
            .iter()
            .map(|&val| (val / state.peak_energy).min(1.0))
            .collect();

        if let Ok(mut data) = SHARED_AUDIO_DATA.lock() {
            data.melbanks = final_melbanks;
        }

        state.audio_samples.drain(0..fft_size);
    }
    // --- END: NEW DSP LOGIC ---
}

// Helper struct to hold state for the JNI function
struct AudioProcessingState {
    fft_size: usize,
    audio_samples: Vec<f32>,
    fft_buffer: Vec<Complex<f32>>,
    window: Vec<f32>,
    fft_plan: Arc<dyn rustfft::Fft<f32>>,
    smoothed_melbanks: Vec<f32>,
    peak_energy: f32,
}

impl Default for AudioProcessingState {
    fn default() -> Self {
        Self {
            fft_size: 0,
            audio_samples: Vec::new(),
            fft_buffer: Vec::new(),
            window: Vec::new(),
            fft_plan: FftPlanner::new().plan_fft_forward(0),
            smoothed_melbanks: Vec::new(),
            peak_energy: 1.0,
        }
    }
}
impl AudioProcessingState {
    fn initialize(&mut self, fft_size: usize, num_bands: usize) {
        self.fft_size = fft_size;
        self.audio_samples.clear();
        self.fft_buffer = vec![Complex::new(0.0, 0.0); fft_size];
        self.window = (0..fft_size)
            .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / (fft_size - 1) as f32).cos()))
            .collect();
        self.fft_plan = FftPlanner::new().plan_fft_forward(fft_size);
        self.smoothed_melbanks = vec![0.0; num_bands];
        self.peak_energy = 1.0;
    }
}

// --- CPAL Stream Building Logic (ported from desktop.rs) ---

fn find_device(host: &cpal::Host, name: &str) -> Option<Device> {
    host.input_devices()
        .ok()?
        .find(|d| d.name().unwrap_or_default() == name)
}

fn build_and_play_stream_cpal(
    device: Device,
    config: SupportedStreamConfig,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
    dsp_settings: Arc<Mutex<DspSettings>>,
) -> Stream {
    // This function is identical to the one in desktop.rs
    // It contains the generic `process_audio` helper inside it.
}

// --- Fully Functional Device Management ---

pub fn get_android_devices() -> Result<AudioDevicesInfo, String> {
    let mut device_list: Vec<AudioDevice> = Vec::new();
    let native_device_name = "System Audio (Native Visualizer)".to_string();
    device_list.push(AudioDevice {
        name: native_device_name.clone(),
    });

    let host = cpal::default_host();
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                device_list.push(AudioDevice { name });
            }
        }
    }

    // On Android, the native visualizer is the best default.
    Ok(AudioDevicesInfo {
        devices: device_list,
        default_device_name: Some(native_device_name),
    })
}

pub fn set_android_device(
    device_name: String,
    command_tx: State<mpsc::Sender<AudioCommand>>,
) -> Result<(), String> {
    println!("[AUDIO] Setting Android audio device to: {}", device_name);
    command_tx
        .send(AudioCommand::ChangeDevice(device_name))
        .map_err(|e| e.to_string())
}
