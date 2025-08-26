pub mod audio;
pub mod effects;
pub mod engine;
pub mod types;
pub mod utils;
pub mod wled;

use crate::effects::{blade_power, scan};
#[cfg(debug_assertions)]
use specta_typescript::Typescript;
use std::sync::mpsc;
use std::thread;
use tauri::Manager;
use tauri_specta::{collect_commands, Builder};

fn configure_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            wled::discover_wled,
            engine::start_effect,
            engine::stop_effect,
            engine::update_effect_settings,
            engine::add_virtual,
            engine::update_virtual,
            engine::remove_virtual,
            engine::add_device,
            engine::remove_device,
            engine::subscribe_to_frames,
            engine::unsubscribe_from_frames,
            engine::set_target_fps,
            engine::get_effect_schema,
            engine::get_available_effects,
            engine::get_devices,
            engine::get_virtuals,
            audio::get_audio_devices,
            audio::set_audio_device,
            audio::get_audio_analysis,
            audio::get_dsp_settings,
            audio::update_dsp_settings,
            engine::get_playback_state,
            engine::toggle_pause
        ])
        .typ::<types::Device>()
        .typ::<types::Virtual>()
        .typ::<types::MatrixCell>()
        .typ::<wled::WledDevice>()
        .typ::<wled::LedsInfo>()
        .typ::<wled::MapInfo>()
        .typ::<audio::AudioDevice>()
        .typ::<audio::DspSettings>()
        .typ::<effects::schema::EffectSetting>()
        .typ::<effects::schema::Control>()
        .typ::<effects::BaseEffectConfig>()
        .typ::<engine::EffectInfo>()
        .typ::<engine::EffectConfig>()
        .typ::<blade_power::BladePowerConfig>()
        .typ::<scan::ScanConfig>()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (engine_command_tx, engine_command_rx) = mpsc::channel::<engine::EngineCommand>();
    let (engine_state_tx, engine_state_rx) = mpsc::channel::<engine::EngineRequest>();
    let (audio_command_tx, audio_command_rx) = mpsc::channel::<audio::AudioCommand>();
    let audio_data = audio::SharedAudioData::default();
    let dsp_settings = audio::SharedDspSettings::default();

    let audio_data_clone_for_thread = audio_data.0.clone();
    let dsp_settings_clone_for_thread = dsp_settings.0.clone();

    // --- START: THE FINAL, CORRECT BUILDER PATTERN ---
    
    // In debug mode, create a builder SOLELY for exporting.
    // This builder is created, used for the export, and then immediately discarded.
    #[cfg(debug_assertions)]
    {
        configure_builder()
            .export(Typescript::default(), "../src/bindings.ts")
            .expect("Failed to export typescript bindings");
    }

    let builder = configure_builder();
    
    // --- END: THE FINAL, CORRECT BUILDER PATTERN ---

    let mut tauri_builder = tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .manage(engine::EngineCommandTx(engine_command_tx))
        .manage(engine::EngineStateTx(engine_state_tx))
        .manage(audio_command_tx)
        .manage(audio_data)
        .manage(dsp_settings)
        .invoke_handler(builder.invoke_handler());

    tauri_builder = tauri_builder.setup(move |app| {
        builder.mount_events(app);
        let state_handle = app.handle().clone();
        let engine_handle = app.handle().clone();

        thread::spawn(move || {
            let audio_data_state = state_handle.state::<audio::SharedAudioData>();
            engine::run_effect_engine(engine_command_rx, engine_state_rx, audio_data_state, engine_handle);
        });

        thread::spawn(move || {
            audio::start_audio_capture(
                audio_command_rx,
                audio_data_clone_for_thread,
                dsp_settings_clone_for_thread,
            );
        });

        Ok(())
    });

    tauri_builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}