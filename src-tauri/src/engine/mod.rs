mod commands;
pub mod generated;
mod handler;
mod renderer;
mod state;

pub use commands::*;
pub use generated::*;
pub use state::*;

use crate::api::ApiCommand;
use crate::audio::SharedAudioData;
use crate::store;
use crate::types::{MatrixCell, Virtual};
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, State};

pub fn run_effect_engine(
    command_rx: mpsc::Receiver<EngineCommand>,
    request_rx: Receiver<EngineRequest>,
    audio_data: State<SharedAudioData>,
    audio_command_tx: Sender<crate::audio::AudioCommand>,
    api_command_tx: Sender<ApiCommand>,
    app_handle: AppHandle,
) {
    let mut engine_state = store::load_engine_state(&app_handle);
    let correct_api_port = engine_state.api_port;
    if let Err(e) = api_command_tx.send(ApiCommand::Restart {
        port: correct_api_port,
    }) {
        eprintln!("[ENGINE] Failed to send initial port to API server: {}", e);
    }

    let mut virtuals: HashMap<String, ActiveVirtual> = engine_state
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
    let mut devices = engine_state.devices.clone();

    for (device_ip, device_config) in &devices {
        let virtual_id = format!("device_{}", device_ip);
        if !virtuals.contains_key(&virtual_id) {
            let matrix_data = vec![(0..device_config.led_count)
                .map(|i| {
                    Some(MatrixCell {
                        device_id: device_ip.clone(),
                        pixel: i,
                    })
                })
                .collect()];
            let device_virtual = Virtual {
                id: virtual_id.clone(),
                name: device_config.name.clone(),
                matrix_data,
                is_device: Some(device_ip.clone()),
            };
            let pixel_count = device_virtual
                .matrix_data
                .iter()
                .flat_map(|row| row.iter())
                .filter(|cell| cell.is_some())
                .count();
            let active_virtual = ActiveVirtual {
                effect: None,
                config: device_virtual,
                pixel_count,
                r_channel: vec![0.0; pixel_count],
                g_channel: vec![0.0; pixel_count],
                b_channel: vec![0.0; pixel_count],
            };
            virtuals.insert(virtual_id, active_virtual);
        }
    }

    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket
        .set_nonblocking(true)
        .expect("Failed to set non-blocking socket");
    let mut frame_count: u8 = 0;
    let mut target_frame_duration = Duration::from_millis(1000 / 60);
    let mut is_paused = false;

    loop {
        let frame_start = Instant::now();

        // --- START: THE FIX ---
        // Declare ONE flag at the top of the loop.
        let mut should_save_state = false;

        while let Ok(request) = request_rx.try_recv() {
            match request {
                EngineRequest::GetVirtuals(responder) => {
                    let virtual_configs: Vec<Virtual> =
                        virtuals.values().map(|v| v.config.clone()).collect();
                    responder.send(virtual_configs).unwrap();
                }
                EngineRequest::GetDevices(responder) => {
                    let device_list: Vec<crate::types::Device> =
                        devices.values().cloned().collect();
                    responder.send(device_list).unwrap();
                }
                EngineRequest::GetPlaybackState(responder) => {
                    responder.send(PlaybackState { is_paused }).unwrap();
                }
                EngineRequest::GetDspSettings(responder) => {
                    responder.send(engine_state.dsp_settings.clone()).unwrap();
                }
                EngineRequest::GetPresets(effect_id, responder) => {
                    let user_presets = engine_state
                        .effect_presets
                        .get(&effect_id)
                        .cloned()
                        .unwrap_or_default();
                    let built_in_presets = get_built_in_presets_for_effect(&effect_id);
                    responder
                        .send(PresetCollection {
                            user: user_presets,
                            built_in: built_in_presets,
                        })
                        .unwrap();
                }
                EngineRequest::GetScenes(responder) => {
                    let scene_list = engine_state.scenes.values().cloned().collect();
                    responder.send(scene_list).unwrap();
                }
                EngineRequest::GetFullState(responder) => {
                    responder.send(engine_state.clone()).unwrap();
                }
                EngineRequest::SavePreset {
                    effect_id,
                    preset_name,
                    settings,
                    responder,
                } => {
                    engine_state
                        .effect_presets
                        .entry(effect_id)
                        .or_default()
                        .insert(preset_name, settings);
                    should_save_state = true; // Set the single flag
                    responder.send(()).unwrap();
                }
                EngineRequest::DeletePreset {
                    effect_id,
                    preset_name,
                    responder,
                } => {
                    if let Some(effect_map) = engine_state.effect_presets.get_mut(&effect_id) {
                        if effect_map.remove(&preset_name).is_some() {
                            should_save_state = true; // Set the single flag
                        }
                    }
                    responder.send(()).unwrap();
                }
            }
        }

        while let Ok(command) = command_rx.try_recv() {
            if let EngineCommand::SetTargetFps { fps } = command {
                if fps > 0 {
                    target_frame_duration = Duration::from_millis(1000 / fps as u64);
                }
            } else {
                // The handler now correctly contributes to the single flag
                should_save_state |= handler::handle_command(
                    command,
                    &mut engine_state,
                    &mut virtuals,
                    &mut devices,
                    &mut is_paused,
                    &audio_command_tx,
                    &api_command_tx,
                    &app_handle,
                );
            }
        }

        if should_save_state {
            // This will now correctly fire for both requests and commands.
            println!(
                "[SAVE STATE] Writing to disk. User presets for 'blade_power': {:?}",
                engine_state.effect_presets.get("blade_power")
            );
            engine_state.devices = devices.clone();
            engine_state.virtuals = virtuals
                .iter()
                .map(|(id, v)| (id.clone(), v.config.clone()))
                .collect();
            store::save_engine_state(&app_handle, &engine_state);
        }
        // --- END: THE FIX ---

        if !is_paused {
            frame_count = frame_count.wrapping_add(1);
            renderer::render_frame(
                &mut virtuals,
                &audio_data,
                &devices,
                &socket,
                frame_count,
                &app_handle,
            );
        }

        let frame_duration = frame_start.elapsed();
        if let Some(sleep_duration) = target_frame_duration.checked_sub(frame_duration) {
            thread::sleep(sleep_duration);
        }
    }
}
