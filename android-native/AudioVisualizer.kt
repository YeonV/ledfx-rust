// src-tauri/gen/android/app/src/main/java/com/blade/ledfx_rust/AudioVisualizer.kt

package com.blade.ledfx_rust

import android.media.audiofx.Visualizer
import android.util.Log

class AudioVisualizer {
    private var visualizer: Visualizer? = null

    // This is the native Rust function we will call from Kotlin.
    private external fun onFftDataCapture(fft: ByteArray, samplingRate: Int)

    fun start() {
        Log.d("LedFxRust", "Attempting to start audio visualizer...")
        try {
            // A session ID of 0 captures the global audio mix.
            visualizer = Visualizer(0).apply {
                captureSize = Visualizer.getCaptureSizeRange()[1] // Get the max capture size
                
                setDataCaptureListener(object : Visualizer.OnDataCaptureListener {
                    override fun onWaveFormDataCapture(
                        visualizer: Visualizer,
                        waveform: ByteArray,
                        samplingRate: Int
                    ) {
                        // We don't need the waveform data.
                    }

                    override fun onFftDataCapture(
                        visualizer: Visualizer,
                        fft: ByteArray,
                        samplingRate: Int
                    ) {
                        // This is the magic. Call our native Rust function with the FFT data.
                        onFftDataCapture(fft, samplingRate)
                    }
                }, Visualizer.getMaxCaptureRate() / 2, false, true) // Capture FFT, not waveform

                enabled = true
            }
            Log.d("LedFxRust", "Audio visualizer started successfully.")
        } catch (e: Exception) {
            Log.e("LedFxRust", "Error initializing Visualizer: ${e.message}")
        }
    }

    fun stop() {
        visualizer?.enabled = false
        visualizer?.release()
        visualizer = null
        Log.d("LedFxRust", "Audio visualizer stopped.")
    }
}