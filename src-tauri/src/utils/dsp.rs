use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Type, Clone, Debug)]
pub struct BladePlusParams {
    pub log_base: f32,
    pub multiplier: f32,
    pub divisor: f32,
}

#[derive(Serialize, Deserialize, Type, Clone, Debug, Default)]
pub enum FilterbankType {
    #[default]
    Balanced,
    Precision,
    Vocal,
    Blade,
    BladePlus(BladePlusParams),
}

fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}
fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10.0f32.powf(mel / 2595.0) - 1.0)
}
fn hz_to_blade(hz: f32) -> f32 {
    3700.0 * (1.0 + (hz / 230.0)).log(12.0)
}
fn blade_to_hz(blade: f32) -> f32 {
    230.0 * (12.0f32.powf(blade / 3700.0) - 1.0)
}
fn hz_to_vocal(hz: f32) -> f32 {
    3340.0 * (1.0 + (hz / 250.0)).log(9.0)
}
fn vocal_to_hz(vocal: f32) -> f32 {
    250.0 * (9.0f32.powf(vocal / 3340.0) - 1.0)
}

pub fn generate_filterbank(
    fft_size: usize,
    sample_rate: u32,
    num_bands: usize,
    min_freq: f32,
    max_freq: f32,
    filter_type: &FilterbankType,
) -> Vec<Vec<(usize, f32)>> {
    let hz_points: Vec<f32> = get_hz_points(num_bands, min_freq, max_freq, filter_type);

    let mut fft_bins: Vec<usize> = hz_points
        .into_iter()
        .map(|hz| (hz * (fft_size as f32 / sample_rate as f32)).floor() as usize)
        .collect();
    for i in 1..fft_bins.len() {
        fft_bins[i] = fft_bins[i].max(fft_bins[i - 1] + 1);
    }

    let mut filters = Vec::with_capacity(num_bands);

    for i in 0..num_bands {
        let mut filter = Vec::new();
        let start_bin = fft_bins[i];
        let center_bin = fft_bins[i + 1];
        let end_bin = fft_bins[i + 2];

        for k in start_bin..center_bin {
            if center_bin > start_bin {
                let weight = (k - start_bin) as f32 / (center_bin - start_bin) as f32;
                filter.push((k, weight));
            }
        }

        for k in center_bin..end_bin {
            if end_bin > center_bin {
                let weight = (end_bin - k) as f32 / (end_bin - center_bin) as f32;
                filter.push((k, weight));
            }
        }
        filters.push(filter);
    }

    filters
}

fn get_hz_points(
    num_bands: usize,
    min_freq: f32,
    max_freq: f32,
    filter_type: &FilterbankType,
) -> Vec<f32> {
    match filter_type {
        FilterbankType::Balanced => {
            let min_mel = hz_to_mel(min_freq);
            let max_mel = hz_to_mel(max_freq);
            (0..=num_bands + 1)
                .map(|i| min_mel + i as f32 * (max_mel - min_mel) / (num_bands + 1) as f32)
                .map(mel_to_hz)
                .collect()
        }
        FilterbankType::Precision => (0..=num_bands + 1)
            .map(|i| min_freq + i as f32 * (max_freq - min_freq) / (num_bands + 1) as f32)
            .collect(),
        FilterbankType::Blade => {
            let min_blade = hz_to_blade(min_freq);
            let max_blade = hz_to_blade(max_freq);
            (0..=num_bands + 1)
                .map(|i| min_blade + i as f32 * (max_blade - min_blade) / (num_bands + 1) as f32)
                .map(blade_to_hz)
                .collect()
        }
        FilterbankType::Vocal => {
            let min_vocal = hz_to_vocal(min_freq);
            let max_vocal = hz_to_vocal(max_freq);
            (0..=num_bands + 1)
                .map(|i| min_vocal + i as f32 * (max_vocal - min_vocal) / (num_bands + 1) as f32)
                .map(vocal_to_hz)
                .collect()
        }
        FilterbankType::BladePlus(params) => {
            let hz_to_custom =
                |hz: f32| params.multiplier * (1.0 + (hz / params.divisor)).log(params.log_base);
            let custom_to_hz = |custom: f32| {
                params.divisor * (params.log_base.powf(custom / params.multiplier) - 1.0)
            };
            let min_custom = hz_to_custom(min_freq);
            let max_custom = hz_to_custom(max_freq);
            (0..=num_bands + 1)
                .map(|i| min_custom + i as f32 * (max_custom - min_custom) / (num_bands + 1) as f32)
                .map(custom_to_hz)
                .collect()
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn calculate_center_frequencies(
    num_bands: u32,
    min_freq: f32,
    max_freq: f32,
    filter_type: FilterbankType,
) -> Result<Vec<f32>, String> {
    let hz_points = get_hz_points(num_bands as usize, min_freq, max_freq, &filter_type);
    Ok(hz_points[1..=num_bands as usize].to_vec())
}

// Gaussian blur implementation is unchanged
// ...
// --- SNIP ---
// ...

/// Generates a 1D Gaussian kernel.
fn create_gaussian_kernel(sigma: f32, array_len: usize) -> Vec<f32> {
    if sigma <= 0.0 {
        return vec![1.0];
    }

    let radius = (4.0 * sigma).ceil() as usize;
    let radius = radius.min((array_len.saturating_sub(1)) / 2).max(1);
    let size = 2 * radius + 1;
    let mut kernel = Vec::with_capacity(size);
    let mut sum = 0.0;

    let sigma_sq_2 = 2.0 * sigma * sigma;

    for i in 0..size {
        let x = (i as i32 - radius as i32) as f32;
        let val = (-x * x / sigma_sq_2).exp();
        kernel.push(val);
        sum += val;
    }

    // Normalize the kernel
    for val in kernel.iter_mut() {
        *val /= sum;
    }

    kernel
}

/// Applies a 1D Gaussian blur to a mutable slice of f32 values.
pub fn gaussian_blur_1d(data: &mut [f32], sigma: f32) {
    if sigma <= 0.0 || data.len() < 3 {
        return;
    }

    let kernel = create_gaussian_kernel(sigma, data.len());
    let kernel_radius = (kernel.len() / 2) as isize;
    let data_len = data.len() as isize;
    let original = data.to_vec(); // Create a copy for reading

    for i in 0..data_len {
        let mut sum = 0.0;
        for k_idx in 0..kernel.len() {
            let k_offset = k_idx as isize - kernel_radius;
            let data_idx = i + k_offset;

            // Simple "mirroring" for edge handling
            let read_idx = if data_idx < 0 {
                -data_idx
            } else if data_idx >= data_len {
                data_len - 1 - (data_idx - (data_len - 1))
            } else {
                data_idx
            } as usize;

            sum += original[read_idx.min(data.len() - 1)] * kernel[k_idx];
        }
        data[i as usize] = sum;
    }
}
