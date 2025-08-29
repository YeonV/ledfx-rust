use super::{AudioAnalysisData, AudioCommand, DspSettings, SharedDspSettings};
use dasp_sample::{Sample, ToSample};
use jni::objects::{JByteArray, JClass};
use jni::sys::jint;
use jni::JNIEnv;
use once_cell::sync::Lazy;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::sync::{mpsc, Arc, Mutex};

// --- START: GLOBAL STATE FOR JNI ---
// We need static state that the JNI function can access.
static SHARED_AUDIO_DATA: Lazy<Arc<Mutex<AudioAnalysisData>>> = Lazy::new(Default::default);
static SHARED_DSP_SETTINGS: Lazy<SharedDspSettings> = Lazy::new(Default::default);
// --- END: GLOBAL STATE FOR JNI ---

// This function is called from lib.rs to link the engine's state to our static state.
pub fn run_android_capture(
    _command_rx: mpsc::Receiver<AudioCommand>, // Commands are not yet handled on Android
    audio_data: Arc<Mutex<AudioAnalysisData>>,
    dsp_settings: Arc<Mutex<DspSettings>>,
) {
    // Connect the shared state from the main app to our static JNI-accessible state.
    let mut global_data = SHARED_AUDIO_DATA.lock().unwrap();
    *global_data = audio_data.lock().unwrap().clone();

    let mut global_settings = SHARED_DSP_SETTINGS.0.lock().unwrap();
    *global_settings = dsp_settings.lock().unwrap().clone();

    println!("[AUDIO] Android capture context initialized.");
}

// --- JNI FUNCTION: The entry point for audio data from Android ---
#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_blade_ledfxrust_AudioVisualizer_onPcmDataCapture(
    env: JNIEnv,
    _class: JClass,
    pcm_data: JByteArray, // We now expect raw PCM audio data, not FFT data
    sampling_rate: jint,
) {
    // --- START: NEW DSP LOGIC (ported from desktop.rs) ---
    let settings = SHARED_DSP_SETTINGS.0.lock().unwrap();

    // Check if there's anything to do
    if SHARED_AUDIO_DATA.lock().unwrap().melbanks.is_empty() {
        let num_bands = settings.num_bands as usize;
        if num_bands > 0 {
            SHARED_AUDIO_DATA.lock().unwrap().melbanks = vec![0.0; num_bands];
        } else {
            return;
        }
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

pub fn get_android_devices() -> Result<Vec<AudioDevice>, String> {
    let mut device_list: Vec<AudioDevice> = Vec::new();
    device_list.push(AudioDevice {
        name: "System Audio (Native Visualizer)".to_string(),
    });

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

pub fn set_android_device(
    _device_name: String,
    _command_tx: State<mpsc::Sender<AudioCommand>>,
) -> Result<(), String> {
    println!("set_audio_device on Android will select between Native and CPAL in the future.");
    Ok(())
}
