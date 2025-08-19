// src-tauri/src/audio/desktop.rs

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, SupportedStreamConfig};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::sync::{mpsc, Arc, Mutex};
use tauri::State;
use super::{AudioAnalysisData, AudioDevice, AudioCommand};

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
                    device_list.push(AudioDevice { name: loopback_name });
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
    command_tx.send(AudioCommand::ChangeDevice(device_name)).map_err(|e| e.to_string())
}

pub fn run_desktop_capture(
    command_rx: mpsc::Receiver<AudioCommand>,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
) {
    let host = cpal::default_host();
    let mut current_stream: Option<Stream> = None;
    loop {
        if let Ok(command) = command_rx.recv() {
            match command {
                AudioCommand::ChangeDevice(device_name) => {
                    if let Some(stream) = current_stream.take() {
                        drop(stream);
                    }
                    let is_loopback = cfg!(target_os = "windows") && device_name.starts_with("System Audio (");
                    let device = find_device(&host, &device_name, is_loopback);
                    let config = if is_loopback {
                        device.default_output_config().expect("no default output config")
                    } else {
                        device.default_input_config().expect("no default input config")
                    };
                    let audio_data_clone = audio_data.clone();
                    let stream = build_and_play_stream(device, config, audio_data_clone);
                    current_stream = Some(stream);
                }
            }
        }
    }
}

fn find_device(host: &cpal::Host, name: &str, is_loopback: bool) -> Device {
    if is_loopback {
        if let Some(stripped_name) = name.strip_prefix("System Audio (").and_then(|n| n.strip_suffix(")")) {
            if let Some(d) = host.output_devices().unwrap().find(|d| d.name().unwrap_or_default() == stripped_name) {
                return d;
            }
        }
    }
    if let Some(d) = host.input_devices().unwrap().find(|d| d.name().unwrap_or_default() == name) {
        return d;
    }
    host.default_input_device().expect("no input device available")
}

fn build_and_play_stream(device: Device, config: SupportedStreamConfig, audio_data: Arc<Mutex<AudioAnalysisData>>) -> Stream {
    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            const FFT_SIZE: usize = 256;
            const GAIN: f32 = 30.0;
            if data.len() < FFT_SIZE { return; }
            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(FFT_SIZE);
            let mut buffer: Vec<Complex<f32>> = data[..FFT_SIZE].iter().enumerate().map(|(i, sample)| {
                let window = 0.5 * (1.0 - f32::cos(2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32));
                Complex::new(sample * window, 0.0)
            }).collect();
            fft.process(&mut buffer);
            let magnitudes: Vec<f32> = buffer.iter().map(|c| c.norm()).collect();
            let sum: f32 = magnitudes.iter().sum();
            let volume = (sum / FFT_SIZE as f32 * GAIN).min(1.0);
            let mut analysis_data = audio_data.lock().unwrap();
            analysis_data.volume = volume;
        },
        |err| eprintln!("An error occurred on the audio stream: {}", err),
        None,
    ).expect("Failed to build audio stream");
    stream.play().expect("Failed to play audio stream");
    stream
}