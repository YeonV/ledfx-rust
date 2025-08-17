// src-tauri/src/lib.rs

mod wled;
mod effects;
mod engine;
mod audio;
mod utils;

use std::sync::mpsc;
use std::thread;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (command_tx, command_rx) = mpsc::channel::<engine::EngineCommand>();
    let frame_buffer = engine::SharedFrameBuffer::default();
    let audio_data = audio::SharedAudioData::default();

    // --- THE FIX: Clone the Arc before setting up the app ---
    // This gives us a handle to the data that is safe to move into the audio thread.
    let audio_data_clone_for_thread = audio_data.0.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(command_tx)
        .manage(frame_buffer)
        .manage(audio_data) // The original is managed by Tauri for the engine
        .invoke_handler(tauri::generate_handler![
            wled::discover_wled,
            engine::start_effect,
            engine::stop_effect,
            engine::get_latest_frames
        ])
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Spawn the effect engine thread
            thread::spawn(move || {
                let frame_buffer_state = app_handle.state::<engine::SharedFrameBuffer>();
                let audio_data_state = app_handle.state::<audio::SharedAudioData>();
                engine::run_effect_engine(command_rx, frame_buffer_state, audio_data_state);
            });

            // --- THE FIX: Spawn the audio capture thread with the cloned Arc ---
            thread::spawn(move || {
                audio::run_audio_capture(audio_data_clone_for_thread);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}