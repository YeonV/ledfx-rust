use super::{AudioAnalysisData, AudioCommand, AudioDevice};
use crate::utils::dsp::generate_mel_filterbank;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, SupportedStreamConfig};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::f32::consts::PI;
use std::sync::{mpsc, Arc, Mutex};
use tauri::State;

const SAMPLE_RATE: u32 = 44100;
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
                    let stream = build_and_play_stream(device, config, audio_data_clone);
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
    _config: SupportedStreamConfig,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
) -> Stream {
    let stream_config = cpal::StreamConfig {
        channels: 1,
        sample_rate: cpal::SampleRate(SAMPLE_RATE),
        buffer_size: cpal::BufferSize::Default,
    };
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);
    let mut fft_buffer = vec![Complex::new(0.0, 0.0); FFT_SIZE];
    let mut audio_samples = Vec::with_capacity(FFT_SIZE * 2);

    let window: Vec<f32> = (0..FFT_SIZE)
        .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / (FFT_SIZE - 1) as f32).cos()))
        .collect();
    let mel_filterbank =
        generate_mel_filterbank(FFT_SIZE / 2, SAMPLE_RATE, NUM_BANDS, MIN_FREQ, MAX_FREQ);

    // --- START: ADDED LOGGING ---
    let mut frame_counter: u64 = 0;
    // --- END: ADDED LOGGING ---

    let data_callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
        audio_samples.extend_from_slice(data);

        while audio_samples.len() >= FFT_SIZE {
            frame_counter += 1;
            // Only log once per second (assuming ~43 FFTs/sec) to avoid spam
            let should_log = frame_counter % 43 == 0;
            
            if should_log { println!("\n--- NEW AUDIO CHUNK (Frame {}) ---", frame_counter); }

            // --- LOG 1: RAW INCOMING SAMPLES ---
            if should_log {
                let sample_slice = &audio_samples[0..10]; // Log first 10 samples
                let min = audio_samples.iter().fold(f32::MAX, |a, &b| a.min(b));
                let max = audio_samples.iter().fold(f32::MIN, |a, &b| a.max(b));
                println!("LOG 1: Raw Samples (Min: {:.4}, Max: {:.4}) | First 10: {:?}", min, max, sample_slice);
            }

            for (i, sample) in audio_samples.iter().enumerate().take(FFT_SIZE) {
                fft_buffer[i] = Complex::new(sample * window[i], 0.0);
            }

            // --- LOG 2: SAMPLES AFTER WINDOWING ---
            if should_log {
                let sample_slice = &fft_buffer[0..5]; // Log first 5 complex numbers
                println!("LOG 2: After Hann Window | First 5: {:?}", sample_slice);
            }

            fft.process(&mut fft_buffer);

            // --- LOG 3: FFT OUTPUT ---
            if should_log {
                let fft_slice = &fft_buffer[0..5]; // Log first 5 complex numbers
                println!("LOG 3: FFT Output | First 5: {:?}", fft_slice);
            }

            let magnitudes: Vec<f32> = fft_buffer[0..FFT_SIZE / 2]
                .iter()
                .map(|c| c.norm_sqr())
                .collect();
            
            // --- LOG 4: MAGNITUDES (POWER) ---
            if should_log {
                let mag_slice = &magnitudes[0..10]; // Log first 10 magnitudes
                let sum = magnitudes.iter().sum::<f32>();
                let max = magnitudes.iter().fold(f32::MIN, |a, &b| a.max(b));
                println!("LOG 4: Magnitudes (Sum: {:.4}, Max: {:.4}) | First 10: {:?}", sum, max, mag_slice);
            }

            let melbanks: Vec<f32> = mel_filterbank
                .iter()
                .map(|filter| {
                    let band_energy = filter
                        .iter()
                        .map(|&(bin_index, weight)| magnitudes[bin_index] * weight)
                        .sum::<f32>();
                    band_energy
                })
                .collect();
                
            // --- LOG 5: MELBANK ENERGY (BEFORE SCALING) ---
            if should_log {
                let mel_slice = &melbanks[0..10]; // Log first 10 melbank energies
                let sum = melbanks.iter().sum::<f32>();
                let max = melbanks.iter().fold(f32::MIN, |a, &b| a.max(b));
                println!("LOG 5: Melbank Energy (Sum: {:.4}, Max: {:.4}) | First 10: {:?}", sum, max, mel_slice);
            }
            
            // Re-apply the linear scaling from our last test for the final output
            let final_melbanks: Vec<f32> = melbanks.iter().map(|&e| e.sqrt() * 5.0).collect();

            // --- LOG 6: FINAL MELBANK VALUES ---
            if should_log {
                let final_slice = &final_melbanks[0..10];
                let sum = final_melbanks.iter().sum::<f32>();
                let max = final_melbanks.iter().fold(f32::MIN, |a, &b| a.max(b));
                println!("LOG 6: Final Melbanks (Sum: {:.4}, Max: {:.4}) | First 10: {:?}", sum, max, final_slice);
            }

            if let Ok(mut data) = audio_data.lock() {
                data.melbanks = final_melbanks;
            }

            audio_samples.drain(0..FFT_SIZE);
        }
    };

    let err_callback = |err| eprintln!("an error occurred on the audio stream: {}", err);

    let stream = device
        .build_input_stream(&stream_config, data_callback, err_callback, None)
        .expect("Failed to build audio stream");
    stream.play().expect("Failed to play audio stream");

    stream
}