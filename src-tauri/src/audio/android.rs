use super::{
    AudioAnalysisData, AudioCommand, AudioDevice, AudioDevicesInfo, DspSettings, SharedDspSettings,
};
use crate::audio::shared_processing::build_and_play_stream_shared;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream};
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

static SHARED_AUDIO_DATA: Lazy<Arc<Mutex<AudioAnalysisData>>> = Lazy::new(Default::default);
static SHARED_DSP_SETTINGS: Lazy<SharedDspSettings> = Lazy::new(Default::default);
static IS_NATIVE_CAPTURE_ACTIVE: AtomicBool = AtomicBool::new(true);

pub fn run_android_capture(
    command_rx: mpsc::Receiver<AudioCommand>,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
    dsp_settings: Arc<Mutex<DspSettings>>,
) {
    println!("[ANDROID RUST] run_android_capture thread started.");
    let mut global_data = SHARED_AUDIO_DATA.lock().unwrap();
    *global_data = audio_data.lock().unwrap().clone();
    let mut global_settings = SHARED_DSP_SETTINGS.0.lock().unwrap();
    *global_settings = dsp_settings.lock().unwrap().clone();
    println!("[ANDROID RUST] Global state linked. Defaulting to JNI mode.");

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

                    if let Some(stream) = current_cpal_stream.take() {
                        stream.pause().expect("Failed to pause CPAL stream");
                        drop(stream);
                        println!("[AUDIO] Stopped existing CPAL stream.");
                    }

                    if device_name.contains("Native") {
                        IS_NATIVE_CAPTURE_ACTIVE.store(true, Ordering::SeqCst);
                        println!("[AUDIO] Switched to JNI Native Visualizer mode.");
                    } else {
                        IS_NATIVE_CAPTURE_ACTIVE.store(false, Ordering::SeqCst);
                        println!("[AUDIO] Switched to CPAL device: {}", device_name);

                        if let Some(device) = find_device(&host, &device_name) {
                            if let Ok(config) = device.default_input_config() {
                                let stream = build_and_play_stream_shared(
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
                    println!("[AUDIO] Android received RestartStream command (no-op, frontend should re-select device).");
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

struct AudioProcessingState {
    fft_size: usize,
    num_bands: usize,
    audio_samples: Vec<f32>,
    fft_buffer: Vec<Complex<f32>>,
    window: Vec<f32>,
    fft_plan: Arc<dyn rustfft::Fft<f32>>,
    filterbank: Vec<Vec<(usize, f32)>>,
    smoothed_melbanks: Vec<f32>,
    peak_energy: f32,
}

impl Default for AudioProcessingState {
    fn default() -> Self {
        Self {
            fft_size: 0,
            num_bands: 0,
            audio_samples: Vec::new(),
            fft_buffer: Vec::new(),
            window: Vec::new(),
            fft_plan: FftPlanner::new().plan_fft_forward(0),
            filterbank: Vec::new(),
            smoothed_melbanks: Vec::new(),
            peak_energy: 1.0,
        }
    }
}

impl AudioProcessingState {
    fn initialize(
        &mut self,
        fft_size: usize,
        num_bands: usize,
        sample_rate: u32,
        settings: &DspSettings,
    ) {
        println!(
            "[ANDROID JNI] Initializing DSP state. FFT Size: {}, Bands: {}",
            fft_size, num_bands
        );
        self.fft_size = fft_size;
        self.num_bands = num_bands;
        self.audio_samples.clear();
        self.audio_samples.reserve(fft_size * 2);
        self.fft_buffer = vec![Complex::new(0.0, 0.0); fft_size];
        self.window = (0..fft_size)
            .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / (fft_size - 1) as f32).cos()))
            .collect();
        self.fft_plan = FftPlanner::new().plan_fft_forward(fft_size);
        self.smoothed_melbanks = vec![0.0; num_bands];
        self.peak_energy = 1.0;
        self.filterbank = crate::utils::dsp::generate_filterbank(
            fft_size / 2,
            sample_rate,
            num_bands,
            settings.min_freq,
            settings.max_freq,
            &settings.filterbank_type,
        );
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_com_blade_ledfxrust_AudioVisualizer_onPcmDataCapture(
    env: JNIEnv,
    _class: JClass,
    pcm_data: JByteArray,
    sampling_rate: jint,
) {
    if !IS_NATIVE_CAPTURE_ACTIVE.load(Ordering::SeqCst) {
        return;
    }

    static PROCESSING_STATE: Lazy<Mutex<AudioProcessingState>> = Lazy::new(Default::default);
    let mut state = PROCESSING_STATE.lock().unwrap();
    let settings = SHARED_DSP_SETTINGS.0.lock().unwrap();

    let fft_size = settings.fft_size as usize;
    if state.fft_size != fft_size {
        state.initialize(
            fft_size,
            settings.num_bands as usize,
            sampling_rate as u32,
            &settings,
        );
    }

    let pcm_bytes = env.convert_byte_array(pcm_data).unwrap();
    let samples_i16: Vec<i16> = pcm_bytes
        .chunks_exact(2)
        .map(|a| i16::from_le_bytes([a[0], a[1]]))
        .collect();

    let source_sample_rate = sampling_rate as u32;
    let target_sample_rate = settings.sample_rate;

    let mono_samples_iterator: Box<dyn Iterator<Item = f32>> =
        if let Some(target_rate) = target_sample_rate {
            if target_rate == source_sample_rate {
                Box::new(samples_i16.into_iter().map(|s| s.to_sample::<f32>()))
            } else {
                println!(
                    "[ANDROID JNI] Resampling from {}Hz to {}Hz",
                    source_sample_rate, target_rate
                );
                let source_signal =
                    signal::from_iter(samples_i16.into_iter().map(|s| [s.to_sample::<f32>()]));
                let linear = Linear::new([0.0], [0.0]);
                let converter = signal::interpolate::Converter::from_hz_to_hz(
                    source_signal,
                    linear,
                    source_sample_rate as f64,
                    target_rate as f64,
                );
                let resampled_mono: Vec<f32> =
                    converter.until_exhausted().map(|frame| frame[0]).collect();
                Box::new(resampled_mono.into_iter())
            }
        } else {
            Box::new(samples_i16.into_iter().map(|s| s.to_sample::<f32>()))
        };

    state.audio_samples.extend(mono_samples_iterator);

    let AudioProcessingState {
        audio_samples,
        fft_buffer,
        window,
        fft_plan,
        filterbank,
        smoothed_melbanks,
        peak_energy,
        num_bands,
        .. // Ignore fft_size as we already have it
    } = &mut *state;

    while audio_samples.len() >= fft_size {
        // Now we are borrowing the fields individually, which is safe.
        for (i, sample) in audio_samples.iter().take(fft_size).enumerate() {
            fft_buffer[i] = Complex::new(sample * window[i], 0.0);
        }

        fft_plan.process(fft_buffer);

        let magnitudes: Vec<f32> = fft_buffer[0..fft_size / 2]
            .iter()
            .map(|c| c.norm_sqr().sqrt())
            .collect();
        let raw_melbanks: Vec<f32> = filterbank
            .iter()
            .map(|filter| filter.iter().map(|&(bin, w)| magnitudes[bin] * w).sum())
            .collect();

        for i in 0..*num_bands {
            smoothed_melbanks[i] = (smoothed_melbanks[i] * settings.smoothing_factor)
                + (raw_melbanks[i] * (1.0 - settings.smoothing_factor));
        }

        let current_max_energy = smoothed_melbanks
            .iter()
            .fold(0.0f32, |max, &val| val.max(max));

        if current_max_energy > *peak_energy {
            *peak_energy = *peak_energy * (1.0 - settings.agc_attack)
                + current_max_energy * settings.agc_attack;
        } else {
            *peak_energy =
                *peak_energy * (1.0 - settings.agc_decay) + current_max_energy * settings.agc_decay;
        }
        *peak_energy = peak_energy.max(1e-4);

        let final_melbanks: Vec<f32> = smoothed_melbanks
            .iter()
            .map(|&val| (val / *peak_energy).min(1.0))
            .collect();

        if let Ok(mut data) = SHARED_AUDIO_DATA.lock() {
            data.melbanks = final_melbanks;
        }

        audio_samples.drain(0..fft_size);
    }
}

fn find_device(host: &cpal::Host, name: &str) -> Option<Device> {
    host.input_devices()
        .ok()?
        .find(|d| d.name().unwrap_or_default() == name)
}

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
