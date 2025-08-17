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
            engine::unsubscribe_from_frames,
            engine::set_target_fps
        ])
        .setup(|app| {
            let state_handle = app.handle().clone();
            let engine_handle = app.handle().clone();

            thread::spawn(move || {
                let audio_data_state = state_handle.state::<audio::SharedAudioData>();
                engine::run_effect_engine(command_rx, audio_data_state, engine_handle);
            });

            thread::spawn(move || {
                audio::run_audio_capture(audio_data_clone_for_thread);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}