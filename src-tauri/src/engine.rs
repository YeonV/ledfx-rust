use crate::audio::{DspSettings, SharedAudioData};
use crate::effects::Effect;
use crate::store;
use crate::types::{Device, MatrixCell, Virtual};
use crate::utils::{colors, ddp, dsp};
use serde::Serialize;
use specta::Type;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};
mod generated;
pub use generated::*;


struct ActiveVirtual {
    effect: Option<Box<dyn Effect>>,
    config: Virtual,
    pixel_count: usize,
    r_channel: Vec<f32>,
    g_channel: Vec<f32>,
    b_channel: Vec<f32>,
}

#[derive(Serialize, Type, Clone)]
pub struct EffectInfo {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, Type, Clone)]
pub struct PlaybackState {
    pub is_paused: bool,
}

pub struct EngineStateTx(pub Sender<EngineRequest>);
pub enum EngineRequest {
    GetVirtuals(Sender<Vec<Virtual>>),
    GetDevices(Sender<Vec<Device>>),
    GetDspSettings(Sender<DspSettings>),
    GetPlaybackState(Sender<PlaybackState>),
}

pub enum EngineCommand {
    StartEffect {
        virtual_id: String,
        config: EffectConfig,
    },
    StopEffect {
        virtual_id: String,
    },
    UpdateSettings {
        virtual_id: String,
        settings: EffectConfig,
    },
    AddVirtual {
        config: Virtual,
    },
    UpdateVirtual {
        config: Virtual,
    },
    RemoveVirtual {
        virtual_id: String,
    },
    AddDevice {
        config: Device,
    },
    RemoveDevice {
        device_ip: String,
    },
    SetTargetFps {
        fps: u32,
    },
    UpdateDspSettings {
        settings: DspSettings,
    },
    // --- START: NEW COMMAND (Master Plan v2.2) ---
    RestartAudioCapture,
    // --- END: NEW COMMAND ---
    TogglePause,
    ReloadState,
}

pub struct EngineCommandTx(pub mpsc::Sender<EngineCommand>);

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

pub fn run_effect_engine(
    command_rx: mpsc::Receiver<EngineCommand>,
    request_rx: Receiver<EngineRequest>,
    audio_data: State<SharedAudioData>,
    audio_command_tx: mpsc::Sender<crate::audio::AudioCommand>,
    app_handle: AppHandle,
) {
    let mut engine_state = store::load_engine_state(&app_handle);
    let mut virtuals: HashMap<String, ActiveVirtual> = engine_state
        .virtuals
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
    let mut devices = engine_state.devices;
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let mut frame_count: u8 = 0;
    let mut target_frame_duration = Duration::from_millis(1000 / 60);
    let mut is_paused = false;

    loop {
        let frame_start = Instant::now();
        if let Ok(request) = request_rx.try_recv() {
            match request {
                EngineRequest::GetVirtuals(responder) => {
                    let virtual_configs: Vec<Virtual> =
                        virtuals.values().map(|v| v.config.clone()).collect();
                    responder.send(virtual_configs).unwrap();
                }
                EngineRequest::GetDevices(responder) => {
                    let device_list: Vec<Device> = devices.values().cloned().collect();
                    responder.send(device_list).unwrap();
                }
                EngineRequest::GetPlaybackState(responder) => {
                    responder.send(PlaybackState { is_paused }).unwrap();
                }
                EngineRequest::GetDspSettings(responder) => {
                    responder.send(engine_state.dsp_settings.clone()).unwrap();
                }
            }
        }

        let mut should_save_state = false;
        while let Ok(command) = command_rx.try_recv() {
            match command {
                EngineCommand::RestartAudioCapture => {
                    println!("[ENGINE] Forwarding RestartStream command to audio thread.");
                    audio_command_tx.send(crate::audio::AudioCommand::RestartStream).unwrap();
                }
                EngineCommand::UpdateDspSettings { settings } => {
                    println!("[ENGINE] Updating DSP settings.");
                    engine_state.dsp_settings = settings.clone();
                    audio_command_tx.send(crate::audio::AudioCommand::UpdateSettings(settings.clone())).unwrap();
                    app_handle.emit("dsp-settings-changed", &settings).unwrap();
                    should_save_state = true;
                }
                // --- START: MODIFIED RELOADSTATE (Master Plan v2.2) ---
                EngineCommand::ReloadState => {
                    println!("[ENGINE] Reloading state from disk.");
                    engine_state = store::load_engine_state(&app_handle);

                    // First, load the custom virtuals from the state file
                    let mut reloaded_virtuals: HashMap<String, ActiveVirtual> = engine_state
                        .virtuals
                        .iter()
                        .filter(|(_, v)| v.is_device.is_none()) 
                        .map(|(id, config)| {
                            let pixel_count = config.matrix_data.iter().flat_map(|row| row.iter()).filter(|cell| cell.is_some()).count();
                            (id.clone(), ActiveVirtual {
                                effect: None,
                                config: config.clone(),
                                pixel_count,
                                r_channel: vec![0.0; pixel_count],
                                g_channel: vec![0.0; pixel_count],
                                b_channel: vec![0.0; pixel_count],
                            })
                        })
                        .collect();

                    // Second, load the devices and create fresh device-virtuals for them
                    devices = engine_state.devices.clone();
                    for (device_ip, device_config) in &devices {
                        let virtual_id = format!("device_{}", device_ip);
                        let matrix_data = vec![(0..device_config.led_count)
                            .map(|i| Some(MatrixCell { device_id: device_ip.clone(), pixel: i }))
                            .collect()];
                        let device_virtual = Virtual {
                            id: virtual_id.clone(),
                            name: device_config.name.clone(),
                            matrix_data,
                            is_device: Some(device_ip.clone()),
                        };
                        let pixel_count = device_virtual.matrix_data.iter().flat_map(|row| row.iter()).filter(|cell| cell.is_some()).count();
                        let active_virtual = ActiveVirtual {
                            effect: None,
                            config: device_virtual,
                            pixel_count,
                            r_channel: vec![0.0; pixel_count],
                            g_channel: vec![0.0; pixel_count],
                            b_channel: vec![0.0; pixel_count],
                        };
                        reloaded_virtuals.insert(virtual_id, active_virtual);
                    }
                    
                    virtuals = reloaded_virtuals;
                    should_save_state = false; // We just loaded, no need to save immediately
                    emit_devices_update(&devices, &app_handle);
                    emit_virtuals_update(&virtuals, &app_handle);
                    app_handle.emit("dsp-settings-changed", &engine_state.dsp_settings).unwrap();
                }
                // --- END: MODIFIED RELOADSTATE ---
                EngineCommand::TogglePause => {
                    is_paused = !is_paused;
                    println!("[ENGINE] Playback state toggled. Paused: {}", is_paused);
                    emit_playback_state_update(is_paused, &app_handle);
                }
                EngineCommand::AddDevice { config } => {
                    let device_ip = config.ip_address.clone();
                    devices.insert(device_ip.clone(), config.clone());
                    let virtual_id = format!("device_{}", device_ip);
                    let matrix_data = vec![(0..config.led_count)
                        .map(|i| { Some(MatrixCell { device_id: device_ip.clone(), pixel: i }) })
                        .collect()];
                    let device_virtual = Virtual {
                        id: virtual_id.clone(),
                        name: config.name.clone(),
                        matrix_data,
                        is_device: Some(device_ip.clone()),
                    };
                    let pixel_count = device_virtual.matrix_data.iter().flat_map(|row| row.iter()).filter(|cell| cell.is_some()).count();
                    virtuals.insert(virtual_id,
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
                    emit_devices_update(&devices, &app_handle);
                    emit_virtuals_update(&virtuals, &app_handle);
                }
                EngineCommand::RemoveDevice { device_ip } => {
                    devices.remove(&device_ip);
                    let virtual_id = format!("device_{}", device_ip);
                    virtuals.remove(&virtual_id);
                    should_save_state = true;
                    emit_devices_update(&devices, &app_handle);
                    emit_virtuals_update(&virtuals, &app_handle);
                }
                EngineCommand::AddVirtual { config } => {
                    let pixel_count = config.matrix_data.iter().flat_map(|row| row.iter()).filter(|cell| cell.is_some()).count();
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
                    emit_virtuals_update(&virtuals, &app_handle);
                }
                EngineCommand::UpdateVirtual { config } => {
                    if let Some(active_virtual) = virtuals.get_mut(&config.id) {
                        let pixel_count = config.matrix_data.iter().flat_map(|row| row.iter()).filter(|cell| cell.is_some()).count();
                        active_virtual.config = config;
                        active_virtual.pixel_count = pixel_count;
                        active_virtual.r_channel.resize(pixel_count, 0.0);
                        active_virtual.g_channel.resize(pixel_count, 0.0);
                        active_virtual.b_channel.resize(pixel_count, 0.0);
                    }
                    should_save_state = true;
                    emit_virtuals_update(&virtuals, &app_handle);
                }
                EngineCommand::RemoveVirtual { virtual_id } => {
                    let mut should_save = false;
                    if let Some(active_virtual) = virtuals.get(&virtual_id) {
                        if let Some(device_ip) = &active_virtual.config.is_device {
                            println!("[ENGINE] Removing device-virtual, also removing device: {}", device_ip);
                            devices.remove(device_ip);
                            emit_devices_update(&devices, &app_handle);
                            should_save = true;
                        }
                    }
                    if virtuals.remove(&virtual_id).is_some() {
                        if !should_save { should_save = true; }
                        emit_virtuals_update(&virtuals, &app_handle);
                    }
                    if should_save { should_save_state = true; }
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
                EngineCommand::UpdateSettings { virtual_id, settings } => {
                    if let Some(active_virtual) = virtuals.get_mut(&virtual_id) {
                        if let Some(effect) = &mut active_virtual.effect {
                            let config_value = config_to_value(settings);
                            effect.update_config(config_value);
                        }
                    }
                }
                EngineCommand::SetTargetFps { fps } => {
                    if fps > 0 {
                        target_frame_duration = Duration::from_millis(1000 / fps as u64);
                    }
                }
            }
        }
        if should_save_state {
            engine_state.devices = devices.clone();
            let custom_virtuals: HashMap<String, Virtual> = virtuals
                .iter()
                .filter(|(_, v)| v.config.is_device.is_none())
                .map(|(id, v)| (id.clone(), v.config.clone()))
                .collect();
            engine_state.virtuals = custom_virtuals;
            store::save_engine_state(&app_handle, &engine_state);
        }

        if !is_paused {
            let latest_audio_data = audio_data.inner().0.lock().unwrap().clone();
            frame_count = frame_count.wrapping_add(1);

            let mut device_buffers: HashMap<String, Vec<u8>> = HashMap::new();
            let mut preview_frames: HashMap<String, Vec<u8>> = HashMap::new();

            for (virtual_id, active_virtual) in &mut virtuals {
                if let Some(effect) = &mut active_virtual.effect {
                    let mut virtual_frame = vec![0u8; active_virtual.pixel_count * 3];
                    effect.render(&latest_audio_data, &mut virtual_frame);
                    let base_config = effect.get_base_config();
                    let pixel_count = active_virtual.pixel_count;

                    for i in 0..pixel_count {
                        active_virtual.r_channel[i] = virtual_frame[i * 3] as f32;
                        active_virtual.g_channel[i] = virtual_frame[i * 3 + 1] as f32;
                        active_virtual.b_channel[i] = virtual_frame[i * 3 + 2] as f32;
                    }

                    if base_config.blur > 0.0 {
                        dsp::gaussian_blur_1d(&mut active_virtual.r_channel, base_config.blur);
                        dsp::gaussian_blur_1d(&mut active_virtual.g_channel, base_config.blur);
                        dsp::gaussian_blur_1d(&mut active_virtual.b_channel, base_config.blur);
                    }

                    if base_config.mirror {
                        let half_len = pixel_count / 2;
                        let r_clone = active_virtual.r_channel.clone();
                        let g_clone = active_virtual.g_channel.clone();
                        let b_clone = active_virtual.b_channel.clone();

                        if base_config.flip {
                            let first_half_r = &r_clone[0..half_len];
                            let first_half_g = &g_clone[0..half_len];
                            let first_half_b = &b_clone[0..half_len];

                            active_virtual.r_channel[0..half_len].copy_from_slice(
                                &first_half_r.iter().rev().cloned().collect::<Vec<f32>>(),
                            );
                            active_virtual.g_channel[0..half_len].copy_from_slice(
                                &first_half_g.iter().rev().cloned().collect::<Vec<f32>>(),
                            );
                            active_virtual.b_channel[0..half_len].copy_from_slice(
                                &first_half_b.iter().rev().cloned().collect::<Vec<f32>>(),
                            );

                            active_virtual.r_channel[pixel_count - half_len..]
                                .copy_from_slice(first_half_r);
                            active_virtual.g_channel[pixel_count - half_len..]
                                .copy_from_slice(first_half_g);
                            active_virtual.b_channel[pixel_count - half_len..]
                                .copy_from_slice(first_half_b);
                        } else {
                            for i in 0..half_len {
                                let mirror_i = pixel_count - 1 - i;
                                active_virtual.r_channel[mirror_i] = r_clone[i];
                                active_virtual.g_channel[mirror_i] = g_clone[i];
                                active_virtual.b_channel[mirror_i] = b_clone[i];
                            }
                        }
                    } else if base_config.flip {
                        active_virtual.r_channel.reverse();
                        active_virtual.g_channel.reverse();
                        active_virtual.b_channel.reverse();
                    }

                    let bg_color = colors::parse_single_color(&base_config.background_color)
                        .unwrap_or([0, 0, 0]);
                    for i in 0..pixel_count {
                        virtual_frame[i * 3] = (active_virtual.r_channel[i] as u8).saturating_add(bg_color[0]);
                        virtual_frame[i * 3 + 1] = (active_virtual.g_channel[i] as u8).saturating_add(bg_color[1]);
                        virtual_frame[i * 3 + 2] = (active_virtual.b_channel[i] as u8).saturating_add(bg_color[2]);
                    }

                    let mut linear_index = 0;
                    for row in &active_virtual.config.matrix_data {
                        for cell in row {
                            if let Some(cell_data) = cell {
                                if let Some(device) = devices.get(&cell_data.device_id) {
                                    let device_buffer = device_buffers.entry(cell_data.device_id.clone()).or_insert_with(|| vec![0; device.led_count as usize * 3]);
                                    let source_idx = linear_index * 3;
                                    let dest_idx = cell_data.pixel as usize * 3;
                                    if dest_idx + 2 < device_buffer.len() && source_idx + 2 < virtual_frame.len() {
                                        device_buffer[dest_idx..dest_idx + 3].copy_from_slice(&virtual_frame[source_idx..source_idx + 3]);
                                    }
                                }
                                linear_index += 1;
                            }
                        }
                    }
                    preview_frames.insert(virtual_id.clone(), virtual_frame);
                }
            }

            for (ip, buffer) in &device_buffers {
                let destination = format!("{}:4048", ip);
                let _ = ddp::send_ddp_packet(&socket, &destination, 0, buffer, frame_count);
            }

            let preview_payload: HashMap<String, Vec<u8>> = preview_frames.into_iter().collect();
            if !preview_payload.is_empty() {
                app_handle.emit("engine-tick", &preview_payload).unwrap();
            }
        }

        let frame_duration = frame_start.elapsed();
        if let Some(sleep_duration) = target_frame_duration.checked_sub(frame_duration) {
            thread::sleep(sleep_duration);
        }
    }
}

// --- START: NEW TAURI COMMAND (Master Plan v2.2) ---
#[tauri::command]
#[specta::specta]
pub fn restart_audio_capture(command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::RestartAudioCapture).map_err(|e| e.to_string())
}
// --- END: NEW TAURI COMMAND ---

#[tauri::command]
#[specta::specta]
pub fn get_playback_state(state_tx: State<EngineStateTx>) -> Result<PlaybackState, String> {
    let (responder_tx, responder_rx) = mpsc::channel();
    state_tx.0.send(EngineRequest::GetPlaybackState(responder_tx)).map_err(|e| e.to_string())?;
    responder_rx.recv().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn toggle_pause(command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::TogglePause).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn start_effect(
    virtual_id: String,
    config: EffectConfig,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx.0.send(EngineCommand::StartEffect { virtual_id, config }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn stop_effect(virtual_id: String, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::StopEffect { virtual_id }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn update_effect_settings(
    virtual_id: String,
    settings: EffectConfig,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx.0.send(EngineCommand::UpdateSettings { virtual_id, settings, }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn add_virtual(config: Virtual, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::AddVirtual { config }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn update_virtual(config: Virtual, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::UpdateVirtual { config }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn remove_virtual(
    virtual_id: String,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx.0.send(EngineCommand::RemoveVirtual { virtual_id }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn add_device(config: Device, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::AddDevice { config }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn remove_device(device_ip: String, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::RemoveDevice { device_ip }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn get_virtuals(state_tx: State<EngineStateTx>) -> Result<Vec<Virtual>, String> {
    let (responder_tx, responder_rx) = mpsc::channel();
    state_tx.0.send(EngineRequest::GetVirtuals(responder_tx)).map_err(|e| e.to_string())?;
    responder_rx.recv().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn get_devices(state_tx: State<EngineStateTx>) -> Result<Vec<Device>, String> {
    let (responder_tx, responder_rx) = mpsc::channel();
    state_tx.0.send(EngineRequest::GetDevices(responder_tx)).map_err(|e| e.to_string())?;
    responder_rx.recv().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn set_target_fps(fps: u32, command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::SetTargetFps { fps }).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn trigger_reload(command_tx: State<EngineCommandTx>) -> Result<(), String> {
    command_tx.0.send(EngineCommand::ReloadState).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn update_dsp_settings(
    settings: DspSettings,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx.0.send(EngineCommand::UpdateDspSettings { settings }).map_err(|e| e.to_string())
}