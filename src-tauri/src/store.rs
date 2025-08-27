use crate::audio::DspSettings;
use crate::effects::schema::DefaultValue; // Import the DefaultValue enum
use crate::types::{Device, Virtual};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct EngineState {
    #[serde(default)]
    pub devices: HashMap<String, Device>,
    #[serde(default)]
    pub virtuals: HashMap<String, Virtual>,
}

// --- START: THE FIX ---
#[derive(Serialize, Deserialize, Type, Clone, Debug, Default)]
pub struct FrontendState {
    #[serde(default)]
    pub selected_effects: HashMap<String, String>,
    // This is now a strongly-typed map that specta can understand.
    #[serde(default)]
    pub effect_settings: HashMap<String, HashMap<String, HashMap<String, DefaultValue>>>,
    #[serde(default)]
    pub dsp_settings: DspSettings,
    #[serde(default)]
    pub selected_audio_device: String,
}
// --- END: THE FIX ---

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FullState {
    #[serde(default)]
    pub engine_state: EngineState,
    #[serde(default)]
    pub frontend_state: FrontendState,
}

fn get_settings_path(app_handle: &AppHandle) -> PathBuf {
    let path = app_handle.path().app_data_dir().unwrap();
    fs::create_dir_all(&path).expect("Failed to create app data directory");
    path.join("settings.json")
}

pub fn load_full_state(app_handle: &AppHandle) -> FullState {
    fs::read_to_string(get_settings_path(app_handle))
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

pub fn save_full_state(app_handle: &AppHandle, state: &FullState) {
    let path = get_settings_path(app_handle);
    let json_string = serde_json::to_string_pretty(state).unwrap();
    fs::write(path, json_string).expect("Failed to write to settings file");
}

// fn get_settings_path(app_handle: &AppHandle) -> PathBuf {
//     let path = app_handle.path().app_data_dir().unwrap();
//     // Ensure the directory exists before trying to write to it
//     fs::create_dir_all(&path).expect("Failed to create app data directory");
//     path.join("engine_settings.json")
// }

pub fn load_engine_state(app_handle: &AppHandle) -> EngineState {
    let path = get_settings_path(app_handle);
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}
pub fn save_engine_state(app_handle: &AppHandle, engine_state: &EngineState) {
    let path = get_settings_path(app_handle);
    let json_string = serde_json::to_string_pretty(engine_state).unwrap();
    fs::write(path, json_string).expect("Failed to write to settings file");
}

#[tauri::command]
#[specta::specta]
pub fn save_frontend_state(
    state: FrontendState,
    app_handle: AppHandle,
) -> Result<(), String> {
    let mut full_state = load_full_state(&app_handle);
    full_state.frontend_state = state;
    save_full_state(&app_handle, &full_state);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn load_frontend_state(app_handle: AppHandle) -> Result<FrontendState, String> {
    Ok(load_full_state(&app_handle).frontend_state)
}

#[tauri::command]
#[specta::specta]
pub fn export_settings(app_handle: AppHandle) -> Result<String, String> {
    let full_state = load_full_state(&app_handle);
    serde_json::to_string_pretty(&full_state).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn import_settings(app_handle: AppHandle, data: String) -> Result<(), String> {
    let _: FullState = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    fs::write(get_settings_path(&app_handle), data).map_err(|e| e.to_string())
}