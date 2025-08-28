use crate::audio::DspSettings;
use crate::types::{Device, Virtual};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use specta::Type;

#[derive(Serialize, Deserialize, Debug, Default, Clone, Type)]
pub struct EngineState {
    #[serde(default)]
    pub devices: HashMap<String, Device>,
    #[serde(default)]
    pub virtuals: HashMap<String, Virtual>,
    #[serde(default)]
    pub dsp_settings: DspSettings,
}

fn get_settings_path(app_handle: &AppHandle) -> PathBuf {
    let path = app_handle.path().app_data_dir().unwrap();
    fs::create_dir_all(&path).expect("Failed to create app data directory");
    path.join("engine_settings.json")
}

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
// Change the return type to a Result
pub fn get_default_engine_state() -> Result<EngineState, String> {
    // Wrap the return value in Ok()
    Ok(EngineState::default())
}

#[tauri::command]
#[specta::specta]
pub fn export_settings(app_handle: AppHandle) -> Result<String, String> {
    let state = load_engine_state(&app_handle);
    serde_json::to_string_pretty(&state).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn import_settings(app_handle: AppHandle, data: String) -> Result<(), String> {
    let path = get_settings_path(&app_handle);
    let _: EngineState = serde_json::from_str(&data).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
}