//! Digital Signal Processing utility functions.
// use std::f32::consts::PI;

/// Converts a frequency in Hertz to the Mel scale.
pub fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

/// Converts a frequency from the Mel scale to Hertz.
pub fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10.0f32.powf(mel / 2595.0) - 1.0)
}

/// Generates a Mel filterbank.
pub fn generate_mel_filterbank(
    fft_size: usize,
    sample_rate: u32,
    num_bands: usize,
    min_freq: f32,
    max_freq: f32,
) -> Vec<Vec<(usize, f32)>> {
    let min_mel = hz_to_mel(min_freq);
    let max_mel = hz_to_mel(max_freq);

    let mel_points: Vec<f32> = (0..=num_bands + 1)
        .map(|i| min_mel + i as f32 * (max_mel - min_mel) / (num_bands + 1) as f32)
        .collect();
    let hz_points: Vec<f32> = mel_points.into_iter().map(mel_to_hz).collect();

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

// --- START: NEW GAUSSIAN BLUR IMPLEMENTATION ---

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
// --- END: NEW GAUSSIAN BLUR IMPLEMENTATION ---
