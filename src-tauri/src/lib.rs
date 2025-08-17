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
    let audio_data = audio::SharedAudioData::default();
    let audio_data_clone_for_thread = audio_data.0.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(command_tx)
        .manage(audio_data)
        .invoke_handler(tauri::generate_handler![
            wled::discover_wled,
            engine::start_effect,
            engine::stop_effect,
            engine::subscribe_to_frames,
            engine::unsubscribe_from_frames
        ])
        .setup(|app| {
            // --- THE FIX ---
            // 1. Get a handle for getting state.
            let state_handle = app.handle().clone();
            // 2. Get a separate handle to move into the engine thread.
            let engine_handle = app.handle().clone();

            // 3. Spawn the effect engine thread.
            thread::spawn(move || {
                // Use the first handle to get the state.
                let audio_data_state = state_handle.state::<audio::SharedAudioData>();
                // Move the second, un-borrowed handle into the engine.
                engine::run_effect_engine(command_rx, audio_data_state, engine_handle);
            });

            // 4. Spawn the audio capture thread with the pre-cloned Arc.
            thread::spawn(move || {
                audio::run_audio_capture(audio_data_clone_for_thread);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}