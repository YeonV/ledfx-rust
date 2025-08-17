// src-tauri/src/audio.rs

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use specta::Type;
// use tauri::State; // <-- REMOVED

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

#[tauri::command]
#[specta::specta]
pub fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let devices = host.input_devices().map_err(|e| e.to_string())?;
    let device_list: Vec<AudioDevice> = devices
        .filter_map(|d| d.name().ok())
        .map(|name| AudioDevice { name })
        .collect();
    Ok(device_list)
}

pub fn run_audio_capture(audio_data: Arc<Mutex<AudioAnalysisData>>) {
    let host = cpal::default_host();
    let device = match host.default_input_device() {
        Some(d) => d,
        None => { eprintln!("FATAL: No audio input device found."); return; }
    };
    let config = match device.default_input_config() {
        Ok(c) => c,
        Err(e) => { eprintln!("FATAL: No default audio config found: {}", e); return; }
    };

    println!("Audio Input Device: {}", device.name().unwrap_or_default());

    let stream = device.build_input_stream(
        &config.into(),
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            const FFT_SIZE: usize = 256;
            const GAIN: f32 = 30.0;
            if data.len() < FFT_SIZE { return; }

            let mut planner = FftPlanner::new();
            let fft = planner.plan_fft_forward(FFT_SIZE);

            let mut buffer: Vec<Complex<f32>> = data[..FFT_SIZE]
                .iter()
                .enumerate()
                .map(|(i, sample)| {
                    let window = 0.5 * (1.0 - f32::cos(2.0 * std::f32::consts::PI * i as f32 / (FFT_SIZE - 1) as f32));
                    Complex::new(sample * window, 0.0)
                })
                .collect();

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
    std::thread::park();
}