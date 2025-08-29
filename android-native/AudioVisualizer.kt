package com.blade.ledfx_rust

import android.media.audiofx.Visualizer
import android.util.Log

class AudioVisualizer {
    private var visualizer: Visualizer? = null

    // This must match the name in android.rs (without the Java_... prefix)
    private external fun onPcmDataCapture(pcm: ByteArray, samplingRate: Int)

    fun start() {
        Log.d("LedFxRust", "Attempting to start audio visualizer...")
        try {
            visualizer = Visualizer(0).apply {
                captureSize = Visualizer.getCaptureSizeRange()[1]
                
                setDataCaptureListener(object : Visualizer.OnDataCaptureListener {
                    // --- THE FIX: We now capture Waveform data ---
                    override fun onWaveFormDataCapture(
                        visualizer: Visualizer,
                        waveform: ByteArray,
                        samplingRate: Int
                    ) {
                        // Pass the raw PCM waveform data to Rust
                        onPcmDataCapture(waveform, samplingRate)
                    }

                    // We no longer need the FFT data from here.
                    override fun onFftDataCapture(
                        visualizer: Visualizer,
                        fft: ByteArray,
                        samplingRate: Int
                    ) {}
                // --- THE FIX: Request WAVEFORM, not FFT ---
                }, Visualizer.getMaxCaptureRate(), true, false) 

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