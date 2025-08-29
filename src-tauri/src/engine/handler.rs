use super::commands::EngineCommand;
use super::generated::{
    config_to_value, create_effect, get_built_in_presets_for_effect, get_effect_id_from_config,
    EffectConfig,
};
use super::state::{ActiveEffectsState, ActiveVirtual, PlaybackState};
use crate::api::ApiCommand;
use crate::audio::AudioCommand;
use crate::store::{self, EngineState, Scene, SceneEffect};
use crate::types::{Device, MatrixCell, Virtual};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use tauri::{AppHandle, Emitter};

fn emit_virtuals_update(virtuals: &HashMap<String, ActiveVirtual>, app_handle: &AppHandle) {
    let virtual_configs: Vec<Virtual> = virtuals.values().map(|v| v.config.clone()).collect();
    app_handle
        .emit("virtuals-changed", &virtual_configs)
        .unwrap();
}
fn emit_devices_update(devices: &HashMap<String, Device>, app_handle: &AppHandle) {
    let device_list: Vec<Device> = devices.values().cloned().collect();
    app_handle.emit("devices-changed", &device_list).unwrap();
}
fn emit_playback_state_update(is_paused: bool, app_handle: &AppHandle) {
    app_handle
        .emit("playback-state-changed", &PlaybackState { is_paused })
        .unwrap();
}
fn emit_scenes_update(scenes: &HashMap<String, Scene>, app_handle: &AppHandle) {
    let scene_list: Vec<Scene> = scenes.values().cloned().collect();
    app_handle.emit("scenes-changed", &scene_list).unwrap();
}
fn emit_active_effects_update(state: &ActiveEffectsState, app_handle: &AppHandle) {
    app_handle.emit("scene-activated", state).unwrap();
}

#[allow(clippy::too_many_arguments)]
pub fn handle_command(
    command: EngineCommand,
    engine_state: &mut EngineState,
    virtuals: &mut HashMap<String, ActiveVirtual>,
    devices: &mut HashMap<String, Device>,
    is_paused: &mut bool,
    audio_command_tx: &Sender<AudioCommand>,
    api_command_tx: &Sender<ApiCommand>,
    app_handle: &AppHandle,
) -> bool {
    let mut should_save_state = false;
    match command {
        EngineCommand::SetApiPort(port) => {
            println!("[ENGINE] Setting API port to {}", port);
            engine_state.api_port = port;
            api_command_tx.send(ApiCommand::Restart { port }).unwrap();
            should_save_state = true;
        }
        EngineCommand::RestartAudioCapture => {
            println!("[ENGINE] Forwarding RestartStream command to audio thread.");
            let _ = audio_command_tx.send(AudioCommand::RestartStream);
        }
        EngineCommand::UpdateDspSettings { settings } => {
            println!("[ENGINE] Updating DSP settings.");
            engine_state.dsp_settings = settings.clone();
            let _ = audio_command_tx.send(AudioCommand::UpdateSettings(settings.clone()));
            app_handle.emit("dsp-settings-changed", &settings).unwrap();
            should_save_state = true;
        }
        EngineCommand::ReloadState => {
            println!("[ENGINE] Reloading state from disk.");
            *engine_state = store::load_engine_state(app_handle);
            *virtuals = engine_state
                .virtuals
                .clone()
                .into_iter()
                .map(|(id, config)| {
                    let pixel_count = config
                        .matrix_data
                        .iter()
                        .flat_map(|row| row.iter())
                        .filter(|cell| cell.is_some())
                        .count();
                    (
                        id,
                        ActiveVirtual {
                            effect: None,
                            config,
                            pixel_count,
                            r_channel: vec![0.0; pixel_count],
                            g_channel: vec![0.0; pixel_count],
                            b_channel: vec![0.0; pixel_count],
                        },
                    )
                })
                .collect();
            *devices = engine_state.devices.clone();
            emit_devices_update(devices, app_handle);
            emit_virtuals_update(virtuals, app_handle);
            app_handle
                .emit("dsp-settings-changed", &engine_state.dsp_settings)
                .unwrap();
        }
        EngineCommand::TogglePause => {
            *is_paused = !*is_paused;
            println!("[ENGINE] Playback state toggled. Paused: {}", is_paused);
            emit_playback_state_update(*is_paused, app_handle);
        }
        EngineCommand::AddDevice { config } => {
            let device_ip = config.ip_address.clone();
            devices.insert(device_ip.clone(), config.clone());
            let virtual_id = format!("device_{}", device_ip);
            let matrix_data = vec![(0..config.led_count)
                .map(|i| {
                    Some(MatrixCell {
                        device_id: device_ip.clone(),
                        pixel: i,
                    })
                })
                .collect()];
            let device_virtual = Virtual {
                id: virtual_id.clone(),
                name: config.name.clone(),
                matrix_data,
                is_device: Some(device_ip.clone()),
            };
            let pixel_count = device_virtual
                .matrix_data
                .iter()
                .flat_map(|row| row.iter())
                .filter(|cell| cell.is_some())
                .count();
            virtuals.insert(
                virtual_id,
                ActiveVirtual {
                    effect: None,
                    config: device_virtual,
                    pixel_count,
                    r_channel: vec![0.0; pixel_count],
                    g_channel: vec![0.0; pixel_count],
                    b_channel: vec![0.0; pixel_count],
                },
            );
            should_save_state = true;
            emit_devices_update(devices, app_handle);
            emit_virtuals_update(virtuals, app_handle);
        }
        EngineCommand::RemoveDevice { device_ip } => {
            devices.remove(&device_ip);
            let virtual_id = format!("device_{}", device_ip);
            virtuals.remove(&virtual_id);
            should_save_state = true;
            emit_devices_update(devices, app_handle);
            emit_virtuals_update(virtuals, app_handle);
        }
        EngineCommand::AddVirtual { config } => {
            let pixel_count = config
                .matrix_data
                .iter()
                .flat_map(|row| row.iter())
                .filter(|cell| cell.is_some())
                .count();
            virtuals.insert(
                config.id.clone(),
                ActiveVirtual {
                    effect: None,
                    config,
                    pixel_count,
                    r_channel: vec![0.0; pixel_count],
                    g_channel: vec![0.0; pixel_count],
                    b_channel: vec![0.0; pixel_count],
                },
            );
            should_save_state = true;
            emit_virtuals_update(virtuals, app_handle);
        }
        EngineCommand::UpdateVirtual { config } => {
            if let Some(active_virtual) = virtuals.get_mut(&config.id) {
                let pixel_count = config
                    .matrix_data
                    .iter()
                    .flat_map(|row| row.iter())
                    .filter(|cell| cell.is_some())
                    .count();
                active_virtual.config = config;
                active_virtual.pixel_count = pixel_count;
                active_virtual.r_channel.resize(pixel_count, 0.0);
                active_virtual.g_channel.resize(pixel_count, 0.0);
                active_virtual.b_channel.resize(pixel_count, 0.0);
            }
            should_save_state = true;
            emit_virtuals_update(virtuals, app_handle);
        }
        EngineCommand::RemoveVirtual { virtual_id } => {
            let mut was_device_virtual = false;
            if let Some(active_virtual) = virtuals.get(&virtual_id) {
                if let Some(device_ip) = &active_virtual.config.is_device {
                    println!(
                        "[ENGINE] Removing device-virtual, also removing device: {}",
                        device_ip
                    );
                    devices.remove(device_ip);
                    emit_devices_update(devices, app_handle);
                    was_device_virtual = true;
                }
            }
            if virtuals.remove(&virtual_id).is_some() {
                emit_virtuals_update(virtuals, app_handle);
                should_save_state = true;
            } else if was_device_virtual {
                should_save_state = true;
            }
        }
        EngineCommand::StartEffect { virtual_id, config } => {
            if let Some(active_virtual) = virtuals.get_mut(&virtual_id) {
                active_virtual.effect = Some(create_effect(config));
            }
        }
        EngineCommand::StopEffect { virtual_id } => {
            if let Some(active_virtual) = virtuals.get_mut(&virtual_id) {
                active_virtual.effect = None;
            }
        }
        EngineCommand::UpdateSettings {
            virtual_id,
            settings,
        } => {
            if let Some(active_virtual) = virtuals.get_mut(&virtual_id) {
                if let Some(effect) = &mut active_virtual.effect {
                    let config_value = config_to_value(settings);
                    effect.update_config(config_value);
                }
            }
        }
        EngineCommand::SetTargetFps { .. } => { /* Handled in main loop */ }
        EngineCommand::SaveScene(scene) => {
            println!("[ENGINE] Saving scene '{}' ({})", scene.name, scene.id);
            engine_state.scenes.insert(scene.id.clone(), scene);
            emit_scenes_update(&engine_state.scenes, app_handle);
            should_save_state = true;
        }
        EngineCommand::DeleteScene(scene_id) => {
            println!("[ENGINE] Deleting scene '{}'", scene_id);
            if engine_state.scenes.remove(&scene_id).is_some() {
                emit_scenes_update(&engine_state.scenes, app_handle);
                should_save_state = true;
            }
        }
        EngineCommand::ActivateScene(scene_id) => {
            if let Some(scene) = engine_state.scenes.get(&scene_id) {
                println!("[ENGINE] Activating scene '{}'", scene.name);
                let mut new_selected_effects: HashMap<String, String> = HashMap::new();
                let mut new_effect_settings: HashMap<String, HashMap<String, EffectConfig>> =
                    HashMap::new();
                let mut new_active_effects: HashMap<String, bool> = HashMap::new();
                for active_virtual in virtuals.values_mut() {
                    active_virtual.effect = None;
                }
                for (virtual_id, scene_effect) in &scene.virtual_effects {
                    if let Some(active_virtual) = virtuals.get_mut(virtual_id) {
                        let effect_config = match scene_effect {
                            SceneEffect::Custom(config) => Some(config.clone()),
                            SceneEffect::Preset(scene_preset) => engine_state
                                .effect_presets
                                .get(&scene_preset.effect_id)
                                .and_then(|presets| presets.get(&scene_preset.preset_name))
                                .cloned()
                                .or_else(|| {
                                    get_built_in_presets_for_effect(&scene_preset.effect_id)
                                        .get(&scene_preset.preset_name)
                                        .cloned()
                                }),
                        };
                        if let Some(config) = effect_config {
                            let effect_id = get_effect_id_from_config(&config);
                            new_selected_effects.insert(virtual_id.clone(), effect_id.clone());
                            new_effect_settings
                                .entry(virtual_id.clone())
                                .or_default()
                                .insert(effect_id.clone(), config.clone());
                            new_active_effects.insert(virtual_id.clone(), true);
                            active_virtual.effect = Some(create_effect(config));
                        }
                    }
                }
                emit_active_effects_update(
                    &ActiveEffectsState {
                        active_scene_id: Some(scene_id.clone()),
                        selected_effects: new_selected_effects,
                        effect_settings: new_effect_settings,
                        active_effects: new_active_effects,
                    },
                    app_handle,
                );
            }
        }
    }
    should_save_state
}
