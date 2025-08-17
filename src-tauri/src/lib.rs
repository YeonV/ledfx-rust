// src-tauri/src/lib.rs

mod wled;
mod effects;
mod engine;

use std::sync::mpsc;
use std::thread;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (command_tx, command_rx) = mpsc::channel::<engine::EngineCommand>();
    let frame_buffer = engine::SharedFrameBuffer::default();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(command_tx)
        .manage(frame_buffer)
        .invoke_handler(tauri::generate_handler![
            wled::discover_wled,
            engine::start_effect,
            engine::stop_effect,
            engine::get_latest_frames
        ])
        .setup(|app| {
            // --- THE FIX ---
            // 1. Get an AppHandle, which is safe to move to other threads.
            let app_handle = app.handle().clone();

            // 2. Move the handle into the new thread.
            thread::spawn(move || {
                // 3. Get the state from the handle *inside* the new thread.
                let frame_buffer_state = app_handle.state::<engine::SharedFrameBuffer>();
                engine::run_effect_engine(command_rx, frame_buffer_state);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}