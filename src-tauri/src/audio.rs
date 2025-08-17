// src-tauri/src/audio.rs

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::sync::{Arc, Mutex};

#[derive(Default, Clone)]
pub struct AudioAnalysisData {
    pub volume: f32,
}

#[derive(Default)]
pub struct SharedAudioData(pub Arc<Mutex<AudioAnalysisData>>);

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
            // --- THE FIX: FFT size must be <= the incoming data size. ---
            const FFT_SIZE: usize = 256;
            const GAIN: f32 = 30.0; // Increased gain slightly for more reactivity

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

            // Only log if the volume is significant to avoid spam.
            // if volume > 0.01 {
            //     println!("AUDIO THREAD: Calculated Volume = {:.4}", volume);
            // }

            let mut analysis_data = audio_data.lock().unwrap();
            analysis_data.volume = volume;
        },
        |err| eprintln!("An error occurred on the audio stream: {}", err),
        None,
    ).expect("Failed to build audio stream");

    stream.play().expect("Failed to play audio stream");
    std::thread::park();
}