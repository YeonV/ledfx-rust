use super::{AudioAnalysisData, DspSettings};
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{Device, SampleFormat, Stream, SupportedStreamConfig};
use dasp::{interpolate::linear::Linear, signal, Signal};
use dasp_sample::{Sample, ToSample};
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::collections::VecDeque;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

// This function is now shared between desktop.rs and android.rs
pub fn build_and_play_stream_shared(
    device: Device,
    config: SupportedStreamConfig,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
    dsp_settings: Arc<Mutex<DspSettings>>,
) -> Stream {
    let initial_settings = dsp_settings.lock().unwrap();
    let fft_size = initial_settings.fft_size as usize;
    let num_bands = initial_settings.num_bands as usize;
    let min_freq = initial_settings.min_freq;
    let max_freq = initial_settings.max_freq;
    let filterbank_type = initial_settings.filterbank_type.clone();
    let target_sample_rate = initial_settings.sample_rate;
    drop(initial_settings);
    let source_sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;

    println!(
        "[AUDIO] Building CPAL stream with FFT size: {}, Bands: {}, Freq Range: {}-{}Hz",
        fft_size, num_bands, min_freq, max_freq
    );
    if let Some(rate) = target_sample_rate {
        println!(
            "[AUDIO] Native sample rate: {}, Resampling to: {}",
            source_sample_rate, rate
        );
    } else {
        println!("[AUDIO] Using native sample rate: {}", source_sample_rate);
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(fft_size);
    let mut fft_buffer = vec![Complex::new(0.0, 0.0); fft_size];
    let mut audio_samples = Vec::with_capacity(fft_size * 2);
    let window: Vec<f32> = (0..fft_size)
        .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / (fft_size - 1) as f32).cos()))
        .collect();
    let final_sample_rate = target_sample_rate.unwrap_or(source_sample_rate);
    let filterbank = crate::utils::dsp::generate_filterbank(
        fft_size / 2,
        final_sample_rate,
        num_bands,
        min_freq,
        max_freq,
        &filterbank_type,
    );
    let mut smoothed_melbanks = vec![0.0; num_bands];
    let mut peak_energy = 1.0;
    let mut delay_buffer: VecDeque<f32> = VecDeque::new();
    let err_callback = |err| eprintln!("an error occurred on stream: {}", err);

    fn process_audio<T: Sample + ToSample<f32>>(
        data: &[T],
        _info: &cpal::InputCallbackInfo,
        channels: usize,
        source_sample_rate: u32,
        target_sample_rate: Option<u32>,
        dsp_settings: &Arc<Mutex<DspSettings>>,
        delay_buffer: &mut VecDeque<f32>,
        audio_samples: &mut Vec<f32>,
        fft_size: usize,
        fft_buffer: &mut Vec<Complex<f32>>,
        fft_plan: &Arc<dyn rustfft::Fft<f32>>,
        window: &Vec<f32>,
        filterbank: &Vec<Vec<(usize, f32)>>,
        num_bands: usize,
        smoothed_melbanks: &mut Vec<f32>,
        peak_energy: &mut f32,
        audio_data: &Arc<Mutex<AudioAnalysisData>>,
    ) {
        let settings = dsp_settings.lock().unwrap();
        let final_sample_rate = target_sample_rate.unwrap_or(source_sample_rate);
        let delay_samples =
            (settings.audio_delay_ms as f32 / 1000.0 * final_sample_rate as f32) as usize;

        let mono_samples: Box<dyn Iterator<Item = f32>> =
            if let Some(target_rate) = target_sample_rate {
                if target_rate == source_sample_rate {
                    Box::new(data.chunks(channels).map(|c| {
                        c.iter().map(|s| s.to_sample::<f32>()).sum::<f32>() / channels as f32
                    }))
                } else {
                    let source_signal =
                        signal::from_iter(data.iter().map(|s| [s.to_sample::<f32>()]));
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
                Box::new(
                    data.chunks(channels).map(|c| {
                        c.iter().map(|s| s.to_sample::<f32>()).sum::<f32>() / channels as f32
                    }),
                )
            };

        delay_buffer.extend(mono_samples);
        while delay_buffer.len() > delay_samples {
            if let Some(delayed_sample) = delay_buffer.pop_front() {
                audio_samples.push(delayed_sample);
            } else {
                break;
            }
        }
        while audio_samples.len() >= fft_size {
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
                .map(|filter| {
                    filter
                        .iter()
                        .map(|&(bin_index, weight)| magnitudes[bin_index] * weight)
                        .sum::<f32>()
                })
                .collect();
            let mut current_max_energy = 0.0f32;
            for i in 0..num_bands {
                smoothed_melbanks[i] = (smoothed_melbanks[i] * settings.smoothing_factor)
                    + (raw_melbanks[i] * (1.0 - settings.smoothing_factor));
                current_max_energy = current_max_energy.max(smoothed_melbanks[i]);
            }
            if current_max_energy > *peak_energy {
                *peak_energy = *peak_energy * (1.0 - settings.agc_attack)
                    + current_max_energy * settings.agc_attack;
            } else {
                *peak_energy = *peak_energy * (1.0 - settings.agc_decay)
                    + current_max_energy * settings.agc_decay;
            }
            *peak_energy = peak_energy.max(1e-4);
            let final_melbanks: Vec<f32> = smoothed_melbanks
                .iter()
                .map(|&val| (val / *peak_energy).min(1.0))
                .collect();
            if let Ok(mut data) = audio_data.lock() {
                data.melbanks = final_melbanks;
            }
            audio_samples.drain(0..fft_size);
        }
    }

    let stream = match config.sample_format() {
        SampleFormat::F32 => device.build_input_stream(
            &config.config(),
            move |data: &[f32], info: &cpal::InputCallbackInfo| {
                process_audio(
                    data,
                    info,
                    channels,
                    source_sample_rate,
                    target_sample_rate,
                    &dsp_settings,
                    &mut delay_buffer,
                    &mut audio_samples,
                    fft_size,
                    &mut fft_buffer,
                    &fft,
                    &window,
                    &filterbank,
                    num_bands,
                    &mut smoothed_melbanks,
                    &mut peak_energy,
                    &audio_data,
                );
            },
            err_callback,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config.config(),
            move |data: &[i16], info: &cpal::InputCallbackInfo| {
                process_audio(
                    data,
                    info,
                    channels,
                    source_sample_rate,
                    target_sample_rate,
                    &dsp_settings,
                    &mut delay_buffer,
                    &mut audio_samples,
                    fft_size,
                    &mut fft_buffer,
                    &fft,
                    &window,
                    &filterbank,
                    num_bands,
                    &mut smoothed_melbanks,
                    &mut peak_energy,
                    &audio_data,
                );
            },
            err_callback,
            None,
        ),
        _ => panic!("Unsupported sample format"),
    }
    .expect("Failed to build audio stream");

    stream.play().expect("Failed to play audio stream");
    stream
}
