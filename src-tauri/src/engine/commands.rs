use super::state::{EngineStateTx, PlaybackState, PresetCollection};
use crate::audio::DspSettings;
use crate::store::Scene;
use crate::types::{Device, Virtual};
use crate::engine::EffectConfig;
use std::sync::mpsc;
use specta::specta;
use tauri::State;

// --- Engine Command Definition ---

pub enum EngineCommand {
    StartEffect { virtual_id: String, config: EffectConfig },
    StopEffect { virtual_id: String },
    UpdateSettings { virtual_id: String, settings: EffectConfig },
    AddVirtual { config: Virtual },
    UpdateVirtual { config: Virtual },
    RemoveVirtual { virtual_id: String },
    AddDevice { config: Device },
    RemoveDevice { device_ip: String },
    SetTargetFps { fps: u32 },
    UpdateDspSettings { settings: DspSettings },
    RestartAudioCapture,
    TogglePause,
    ReloadState,
    SavePreset { effect_id: String, preset_name: String, settings: EffectConfig },
    DeletePreset { effect_id: String, preset_name: String },
    SaveScene(Scene),
    DeleteScene(String),
    ActivateScene(String),
}

pub struct EngineCommandTx(pub mpsc::Sender<EngineCommand>);


// --- Tauri Commands ---

#[tauri::command]
#[specta]
pub fn restart_audio_capture(command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::RestartAudioCapture).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn get_playback_state(state_tx: State<EngineStateTx>) -> Result<PlaybackState, String> {
    let (responder_tx, responder_rx) = mpsc::channel();
    state_tx.0.send(super::state::EngineRequest::GetPlaybackState(responder_tx)).map_err(|e| e.to_string())?;
    responder_rx.recv().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn toggle_pause(command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::TogglePause).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn start_effect(
    virtual_id: String,
    config: EffectConfig,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx.0.send(EngineCommand::StartEffect { virtual_id, config }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn stop_effect(virtual_id: String, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::StopEffect { virtual_id }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn update_effect_settings(
    virtual_id: String,
    settings: EffectConfig,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx.0.send(EngineCommand::UpdateSettings { virtual_id, settings, }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn add_virtual(config: Virtual, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::AddVirtual { config }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn update_virtual(config: Virtual, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::UpdateVirtual { config }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn remove_virtual(
    virtual_id: String,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx.0.send(EngineCommand::RemoveVirtual { virtual_id }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn add_device(config: Device, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::AddDevice { config }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn remove_device(device_ip: String, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::RemoveDevice { device_ip }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn get_virtuals(state_tx: State<EngineStateTx>) -> Result<Vec<Virtual>, String> {
    let (responder_tx, responder_rx) = mpsc::channel();
    state_tx.0.send(super::state::EngineRequest::GetVirtuals(responder_tx)).map_err(|e| e.to_string())?;
    responder_rx.recv().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn get_devices(state_tx: State<EngineStateTx>) -> Result<Vec<Device>, String> {
    let (responder_tx, responder_rx) = mpsc::channel();
    state_tx.0.send(super::state::EngineRequest::GetDevices(responder_tx)).map_err(|e| e.to_string())?;
    responder_rx.recv().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn set_target_fps(fps: u32, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::SetTargetFps { fps }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn update_dsp_settings(
    settings: DspSettings,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx.0.send(EngineCommand::UpdateDspSettings { settings }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn save_preset(
    effect_id: String,
    preset_name: String,
    settings: EffectConfig,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx.0.send(EngineCommand::SavePreset { effect_id, preset_name, settings }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn delete_preset(
    effect_id: String,
    preset_name: String,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx.0.send(EngineCommand::DeletePreset { effect_id, preset_name }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn load_presets(
    effect_id: String,
    engine_state_tx: State<EngineStateTx>,
) -> Result<PresetCollection, String> {
    let (responder_tx, responder_rx) = mpsc::channel();
    engine_state_tx.0.send(super::state::EngineRequest::GetPresets(effect_id, responder_tx)).map_err(|e| e.to_string())?;
    responder_rx.recv().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn save_scene(scene: Scene, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::SaveScene(scene)).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn delete_scene(scene_id: String, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::DeleteScene(scene_id)).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn activate_scene(scene_id: String, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::ActivateScene(scene_id)).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn get_scenes(engine_state_tx: State<EngineStateTx>) -> Result<Vec<Scene>, String> {
    let (responder_tx, responder_rx) = mpsc::channel();
    engine_state_tx.0.send(super::state::EngineRequest::GetScenes(responder_tx)).map_err(|e| e.to_string())?;
    responder_rx.recv().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta]
pub fn trigger_reload(command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::ReloadState).map_err(|e| e.to_string())
}