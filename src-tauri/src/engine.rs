// src-tauri/src/engine.rs

use crate::audio::SharedAudioData;
use crate::effects::{blade, legacy, simple, Effect};
use crate::utils::send_ddp_packet;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::net::UdpSocket;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

// The ActiveEffect no longer needs led_count, as the effect instance
// now holds its own configuration and state.
struct ActiveEffect {
    effect: Box<dyn Effect>,
    led_count: u32,
}

#[tauri::command]
#[specta::specta]
pub fn get_legacy_effect_schema(
    effect_id: String,
) -> Result<Vec<legacy::blade_power::EffectSetting>, String> {
    match effect_id.as_str() {
        "bladepower" => Ok(legacy::blade_power::get_schema()),
        _ => Err(format!("Schema not found for legacy effect: {}", effect_id)),
    }
}

// This is the type-safe enum that the frontend will send.
// It uses the full path to the config structs to be explicit.
#[derive(Deserialize, Serialize, Type, Clone)]
#[serde(tag = "mode", content = "config")]
pub enum EffectConfig {
    #[serde(rename = "legacy")]
    Legacy(crate::effects::legacy::blade_power::BladePowerLegacyConfig),
    #[serde(rename = "blade")]
    Blade(crate::effects::blade::blade_power::BladePowerConfig),
}

// The StartEffect command is now upgraded to use the type-safe config.
pub enum EngineCommand {
    StartEffect {
        ip_address: String,
        led_count: u32,
        effect_id: String,
        config: Option<EffectConfig>,
    },
    StopEffect {
        ip_address: String,
    },
    Subscribe {
        ip_address: String,
    },
    Unsubscribe {
        ip_address: String,
    },
    SetTargetFps {
        fps: u32,
    },
    UpdateSettings {
        ip_address: String,
        settings: Option<EffectConfig>,
    },
}

pub fn run_effect_engine(
    command_rx: mpsc::Receiver<EngineCommand>,
    audio_data: State<SharedAudioData>,
    app_handle: AppHandle,
) {
    let mut active_effects: HashMap<String, ActiveEffect> = HashMap::new();
    let mut subscribed_ips: HashSet<String> = HashSet::new();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let mut frame_count: u8 = 0;
    let mut latest_frames: HashMap<String, Vec<u8>> = HashMap::new();
    let mut target_frame_duration = Duration::from_millis(1000 / 60);
    let mut last_log_time = Instant::now();
    let mut max_volume_this_second = 0.0f32;

    loop {
        let frame_start = Instant::now();
        while let Ok(command) = command_rx.try_recv() {
            match command {
                EngineCommand::StartEffect {
                    ip_address,
                    led_count,
                    effect_id,
                    config,
                } => {
                    if active_effects.remove(&ip_address).is_some() {
                        let black_data = vec![0; (led_count * 3) as usize];
                        let destination = format!("{}:4048", ip_address);
                        let _ = send_ddp_packet(&socket, &destination, 0, &black_data, 0);
                    }

                    let effect: Option<Box<dyn Effect>> = if let Some(config_data) = config {
                        match config_data {
                            EffectConfig::Legacy(conf) => Some(Box::new(
                                legacy::blade_power::BladePowerLegacy::new(conf, led_count),
                            )),
                            EffectConfig::Blade(conf) => Some(Box::new(
                                blade::blade_power::BladePower::new(conf, led_count),
                            )),
                        }
                    } else {
                        // --- THE FIX: Create simple effects from the correct module ---
                        match effect_id.as_str() {
                            "scan" => Some(Box::new(simple::ScanEffect {
                                position: 0,
                                color: [255, 0, 0],
                            })),
                            "scroll" => Some(Box::new(simple::ScrollEffect { hue: 0.0 })),
                            _ => Some(Box::new(simple::RainbowEffect { hue: 0.0 })),
                        }
                    };

                    if let Some(effect) = effect {
                        active_effects.insert(ip_address, ActiveEffect { led_count, effect });
                    } else {
                        eprintln!("Failed to create effect with id '{}'", effect_id);
                    }
                }
                EngineCommand::StopEffect { ip_address } => {
                    if active_effects.remove(&ip_address).is_some() {
                        // This assumes a max LED count for the black frame.
                        // A better solution would be to know the led_count of the stopped effect.
                        let black_data = vec![0; 300 * 3];
                        let destination = format!("{}:4048", ip_address);
                        let _ = send_ddp_packet(&socket, &destination, 0, &black_data, 0);
                        latest_frames.remove(&ip_address);
                    }
                }
                EngineCommand::Subscribe { ip_address } => {
                    subscribed_ips.insert(ip_address);
                }
                EngineCommand::Unsubscribe { ip_address } => {
                    subscribed_ips.remove(&ip_address);
                }
                EngineCommand::SetTargetFps { fps } => {
                    if fps > 0 {
                        target_frame_duration = Duration::from_millis(1000 / fps as u64);
                    }
                }
                EngineCommand::UpdateSettings {
                    ip_address,
                    settings,
                } => {
                    if let Some(active_effect) = active_effects.get_mut(&ip_address) {
                        if let Some(config_enum) = settings {
                            let config_value = match config_enum {
                                EffectConfig::Legacy(config) => {
                                    serde_json::to_value(config).unwrap()
                                }
                                EffectConfig::Blade(config) => {
                                    serde_json::to_value(config).unwrap()
                                }
                            };
                            active_effect.effect.update_settings(config_value);
                        }
                    }
                }
            }
        }

        let latest_audio_data = audio_data.inner().0.lock().unwrap().clone();
        frame_count = frame_count.wrapping_add(1);

        max_volume_this_second = max_volume_this_second.max(latest_audio_data.volume);
        if last_log_time.elapsed() >= Duration::from_secs(1) {
            // println!(
            //     "ENGINE DEBUG: Max volume in last second = {:.4}",
            //     max_volume_this_second
            // );
            max_volume_this_second = 0.0;
            last_log_time = Instant::now();
        }

        for (ip, active_effect) in &mut active_effects {
            // --- THE FIX: Remove all post-processing logic from the engine ---
            let data = active_effect
                .effect
                .render_frame(active_effect.led_count, &latest_audio_data);

            let destination = format!("{}:4048", ip);
            let _ = send_ddp_packet(&socket, &destination, 0, &data, frame_count);
            latest_frames.insert(ip.clone(), data);
        }

        let payload: HashMap<String, Vec<u8>> = latest_frames
            .iter()
            .filter(|(ip, _)| subscribed_ips.contains(*ip))
            .map(|(ip, data)| (ip.clone(), data.clone()))
            .collect();

        if !payload.is_empty() {
            app_handle.emit("engine-tick", &payload).unwrap();
        }

        let frame_duration = frame_start.elapsed();
        if let Some(sleep_duration) = target_frame_duration.checked_sub(frame_duration) {
            thread::sleep(sleep_duration);
        }
    }
}

// The start_effect command is now fully type-safe.
#[tauri::command]
#[specta::specta]
pub fn start_effect(
    ip_address: String,
    led_count: u32,
    effect_id: String,
    config: Option<EffectConfig>,
    command_tx: State<mpsc::Sender<EngineCommand>>,
) -> Result<(), String> {
    command_tx
        .send(EngineCommand::StartEffect {
            ip_address,
            led_count,
            effect_id,
            config,
        })
        .unwrap();
    Ok(())
}

// The other commands remain unchanged from your ground truth.
#[tauri::command]
#[specta::specta]
pub fn stop_effect(
    ip_address: String,
    command_tx: State<mpsc::Sender<EngineCommand>>,
) -> Result<(), String> {
    command_tx
        .send(EngineCommand::StopEffect { ip_address })
        .unwrap();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn subscribe_to_frames(
    ip_address: String,
    command_tx: State<mpsc::Sender<EngineCommand>>,
) -> Result<(), String> {
    command_tx
        .send(EngineCommand::Subscribe { ip_address })
        .unwrap();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn unsubscribe_from_frames(
    ip_address: String,
    command_tx: State<mpsc::Sender<EngineCommand>>,
) -> Result<(), String> {
    command_tx
        .send(EngineCommand::Unsubscribe { ip_address })
        .unwrap();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_target_fps(
    fps: u32,
    command_tx: State<mpsc::Sender<EngineCommand>>,
) -> Result<(), String> {
    command_tx
        .send(EngineCommand::SetTargetFps { fps })
        .unwrap();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn update_effect_settings(
    ip_address: String,
    settings: Option<EffectConfig>,
    command_tx: State<mpsc::Sender<EngineCommand>>,
) -> Result<(), String> {
    command_tx
        .send(EngineCommand::UpdateSettings {
            ip_address,
            settings,
        })
        .unwrap();
    Ok(())
}
