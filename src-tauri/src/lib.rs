// src-tauri/src/lib.rs

pub mod wled;
pub mod effects;
pub mod engine;
pub mod audio;
pub mod utils;

use std::sync::mpsc;
use std::thread;
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};
#[cfg(debug_assertions)]
use specta_typescript::Typescript;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (engine_command_tx, engine_command_rx) = mpsc::channel::<engine::EngineCommand>();
    let (audio_command_tx, audio_command_rx) = mpsc::channel::<audio::AudioCommand>();
    let audio_data = audio::SharedAudioData::default();
    let audio_data_clone_for_thread = audio_data.0.clone();

    let builder = {
        // --- THE FIX: A single, unconditional list of commands ---
        let builder = Builder::<tauri::Wry>::new()
            .commands(collect_commands![
                wled::discover_wled,
                engine::start_effect,
                engine::stop_effect,
                engine::subscribe_to_frames,
                engine::unsubscribe_from_frames,
                engine::set_target_fps,
                audio::get_audio_devices, // This now works on all platforms
                audio::set_audio_device   // This now works on all platforms
            ])
            .typ::<wled::WledDevice>()
            .typ::<wled::LedsInfo>()
            .typ::<wled::MapInfo>()
            .typ::<audio::AudioDevice>();

        #[cfg(debug_assertions)]
        builder
            .export(Typescript::default(), "../src/bindings.ts")
            .expect("Failed to export typescript bindings");

        builder
    };

    let mut tauri_builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(engine_command_tx)
        .manage(audio_command_tx)
        .manage(audio_data)
        .invoke_handler(builder.invoke_handler());

    tauri_builder = tauri_builder.setup(move |app| {
        builder.mount_events(app);
        let state_handle = app.handle().clone();
        let engine_handle = app.handle().clone();

        thread::spawn(move || {
            let audio_data_state = state_handle.state::<audio::SharedAudioData>();
            engine::run_effect_engine(engine_command_rx, audio_data_state, engine_handle);
        });

        thread::spawn(move || {
            audio::start_audio_capture(audio_command_rx, audio_data_clone_for_thread);
        });

        Ok(())
    });

    tauri_builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}