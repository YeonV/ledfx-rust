//! Digital Signal Processing utility functions.

/// Converts a frequency in Hertz to the Mel scale.
pub fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

/// Converts a frequency from the Mel scale to Hertz.
pub fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10.0f32.powf(mel / 2595.0) - 1.0)
}

/// Generates a Mel filterbank.
///
/// # Arguments
/// * `fft_size` - The size of the FFT window.
/// * `sample_rate` - The sample rate of the audio.
/// * `num_bands` - The number of Mel bands to create.
/// * `min_freq` - The minimum frequency for the filterbank.
/// * `max_freq` - The maximum frequency for the filterbank.
///
/// # Returns
/// A vector of filters. Each filter is a vector of `(fft_bin_index, weight)`.
pub fn generate_mel_filterbank(
    fft_size: usize,
    sample_rate: u32,
    num_bands: usize,
    min_freq: f32,
    max_freq: f32,
) -> Vec<Vec<(usize, f32)>> {
    let min_mel = hz_to_mel(min_freq);
    let max_mel = hz_to_mel(max_freq);

    // Calculate the center frequencies of the Mel bands
    let mel_points: Vec<f32> = (0..=num_bands + 1)
        .map(|i| min_mel + i as f32 * (max_mel - min_mel) / (num_bands + 1) as f32)
        .collect();
    let hz_points: Vec<f32> = mel_points.into_iter().map(mel_to_hz).collect();
    let fft_bins: Vec<usize> = hz_points
        .into_iter()
        .map(|hz| (hz * (fft_size as f32 / sample_rate as f32)).floor() as usize)
        .collect();

    let mut filters = Vec::with_capacity(num_bands);

    for i in 0..num_bands {
        let mut filter = Vec::new();
        let start_bin = fft_bins[i];
        let center_bin = fft_bins[i + 1];
        let end_bin = fft_bins[i + 2];

        // Create the rising slope of the triangle
        for k in start_bin..center_bin {
            let weight = (k - start_bin) as f32 / (center_bin - start_bin) as f32;
            filter.push((k, weight));
        }

        // Create the falling slope of the triangle
        for k in center_bin..end_bin {
            let weight = (end_bin - k) as f32 / (end_bin - center_bin) as f32;
            filter.push((k, weight));
        }
        filters.push(filter);
    }

    filters
}