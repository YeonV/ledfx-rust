use super::{AudioAnalysisData, AudioCommand, AudioDevice, DspSettings};
use crate::utils::dsp::generate_mel_filterbank;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, SupportedStreamConfig};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::f32::consts::PI;
use std::sync::{mpsc, Arc, Mutex};
use tauri::State;

const FFT_SIZE: usize = 1024;
const NUM_BANDS: usize = 128;
const MIN_FREQ: f32 = 20.0;
const MAX_FREQ: f32 = 18000.0;

pub fn get_desktop_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let mut device_list: Vec<AudioDevice> = Vec::new();
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                device_list.push(AudioDevice { name });
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(devices) = host.output_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    let loopback_name = format!("System Audio ({})", name);
                    device_list.push(AudioDevice {
                        name: loopback_name,
                    });
                }
            }
        }
    }
    Ok(device_list)
}

pub fn set_desktop_device(
    device_name: String,
    command_tx: State<mpsc::Sender<AudioCommand>>,
) -> Result<(), String> {
    command_tx
        .send(AudioCommand::ChangeDevice(device_name))
        .map_err(|e| e.to_string())
}

pub fn run_desktop_capture(
    command_rx: mpsc::Receiver<AudioCommand>,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
    dsp_settings: Arc<Mutex<DspSettings>>,
) {
    let host = cpal::default_host();
    let mut current_stream: Option<Stream> = None;
    loop {
        if let Ok(command) = command_rx.recv() {
            match command {
                AudioCommand::ChangeDevice(device_name) => {
                    println!("Changing audio device to: {}", device_name);
                    if let Some(stream) = current_stream.take() {
                        drop(stream);
                    }
                    let is_loopback =
                        cfg!(target_os = "windows") && device_name.starts_with("System Audio (");
                    let device = find_device(&host, &device_name, is_loopback);
                    let config = if is_loopback {
                        device
                            .default_output_config()
                            .expect("no default output config")
                    } else {
                        device
                            .default_input_config()
                            .expect("no default input config")
                    };
                    let audio_data_clone = audio_data.clone();
                    let dsp_settings_clone = dsp_settings.clone();
                    let stream = build_and_play_stream(device, config, audio_data_clone, dsp_settings_clone);
                    current_stream = Some(stream);
                }
            }
        }
    }
}

fn find_device(host: &cpal::Host, name: &str, is_loopback: bool) -> Device {
    if is_loopback {
        if let Some(stripped_name) = name
            .strip_prefix("System Audio (")
            .and_then(|n| n.strip_suffix(")"))
        {
            if let Some(d) = host
                .output_devices()
                .unwrap()
                .find(|d| d.name().unwrap_or_default() == stripped_name)
            {
                return d;
            }
        }
    }
    if let Some(d) = host
        .input_devices()
        .unwrap()
        .find(|d| d.name().unwrap_or_default() == name)
    {
        return d;
    }
    host.default_input_device()
        .expect("no input device available")
}


fn build_and_play_stream(
    device: Device,
    config: SupportedStreamConfig,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
    dsp_settings: Arc<Mutex<DspSettings>>,
) -> Stream {
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    let stream_config = config.config();
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);
    let mut fft_buffer = vec![Complex::new(0.0, 0.0); FFT_SIZE];
    let mut audio_samples = Vec::with_capacity(FFT_SIZE * 2);

    let window: Vec<f32> = (0..FFT_SIZE)
        .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / (FFT_SIZE - 1) as f32).cos()))
        .collect();
    
    let mel_filterbank =
        generate_mel_filterbank(FFT_SIZE / 2, sample_rate, NUM_BANDS, MIN_FREQ, MAX_FREQ);

    let mut smoothed_melbanks = vec![0.0; NUM_BANDS];
    let mut peak_energy = 1.0;

    let data_callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        let settings = dsp_settings.lock().unwrap();

        for frame in data.chunks(channels as usize) {
            audio_samples.push(frame.iter().sum::<f32>() / channels as f32);
        }

        while audio_samples.len() >= FFT_SIZE {
            for (i, sample) in audio_samples.iter().enumerate().take(FFT_SIZE) {
                fft_buffer[i] = Complex::new(sample * window[i], 0.0);
            }

            fft.process(&mut fft_buffer);

            let magnitudes: Vec<f32> = fft_buffer[0..FFT_SIZE / 2]
                .iter()
                .map(|c| c.norm_sqr().sqrt())
                .collect();

            let raw_melbanks: Vec<f32> = mel_filterbank
                .iter()
                .map(|filter| {
                    filter
                        .iter()
                        .map(|&(bin_index, weight)| magnitudes[bin_index] * weight)
                        .sum::<f32>()
                })
                .collect();
            
            let mut current_max_energy = 0.0f32;
            for i in 0..NUM_BANDS {
                smoothed_melbanks[i] = (smoothed_melbanks[i] * settings.smoothing_factor)
                    + (raw_melbanks[i] * (1.0 - settings.smoothing_factor));
                current_max_energy = current_max_energy.max(smoothed_melbanks[i]);
            }

            if current_max_energy > peak_energy {
                peak_energy = peak_energy * (1.0 - settings.agc_attack) + current_max_energy * settings.agc_attack;
            } else {
                peak_energy = peak_energy * (1.0 - settings.agc_decay) + current_max_energy * settings.agc_decay;
            }
            peak_energy = peak_energy.max(1e-4);

            let final_melbanks: Vec<f32> = smoothed_melbanks
                .iter()
                .map(|&val| (val / peak_energy).min(1.0))
                .collect();

            if let Ok(mut data) = audio_data.lock() {
                data.melbanks = final_melbanks;
            }

            audio_samples.drain(0..FFT_SIZE);
        }
    };

    let err_callback = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device
        .build_input_stream(&stream_config, data_callback, err_callback, None)
        .expect("Failed to build audio stream");
    stream.play().expect("Failed to play audio stream");

    stream
}