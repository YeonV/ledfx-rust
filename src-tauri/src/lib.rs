// src-tauri/src/lib.rs

pub mod wled;
pub mod effects;
pub mod engine;
pub mod audio;
pub mod utils;

use std::sync::mpsc;
use std::thread;
use tauri::Manager;
// --- THE FIX: Import the correct tools from the new API ---
use tauri_specta::{collect_commands, Builder};
use specta_typescript::Typescript;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (command_tx, command_rx) = mpsc::channel::<engine::EngineCommand>();
    let audio_data = audio::SharedAudioData::default();
    let audio_data_clone_for_thread = audio_data.0.clone();

    // --- THE FIX: Use the Builder pattern ---
    let builder = {
        let builder = Builder::<tauri::Wry>::new()
            .commands(collect_commands![
                wled::discover_wled,
                engine::start_effect,
                engine::stop_effect,
                engine::subscribe_to_frames,
                engine::unsubscribe_from_frames,
                engine::set_target_fps,
                audio::get_audio_devices
            ])
            // Explicitly add the types that are not directly in command signatures
            .typ::<wled::WledDevice>()
            .typ::<wled::LedsInfo>()
            .typ::<wled::MapInfo>();

        #[cfg(debug_assertions)]
        builder
            .export(Typescript::default(), "../src/bindings.ts")
            .expect("Failed to export typescript bindings");

        builder
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(command_tx)
        .manage(audio_data)
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            // Mount events is required by the new API
            builder.mount_events(app);

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