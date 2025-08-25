use crate::audio::SharedAudioData;
use crate::effects::{blade_power, scan, Effect};
use crate::utils::{colors, ddp, dsp};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::net::UdpSocket;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};

struct ActiveEffect {
    effect: Box<dyn Effect>,
    led_count: u32,
    r_channel: Vec<f32>,
    g_channel: Vec<f32>,
    b_channel: Vec<f32>,
}

#[derive(Serialize, Type, Clone)]
pub struct EffectInfo {
    pub id: String,
    pub name: String,
    pub variant: String,
}

#[derive(Deserialize, Serialize, Type, Clone)]
#[serde(tag = "type", content = "config")]
pub enum EffectConfig {
    BladePower(blade_power::BladePowerConfig),
    Scan(scan::ScanConfig),
}

#[tauri::command]
#[specta::specta]
pub fn get_available_effects() -> Result<Vec<EffectInfo>, String> {
    Ok(vec![
        EffectInfo {
            id: "bladepower".to_string(),
            name: "Blade Power".to_string(),
            variant: "BladePower".to_string(),
        },
        EffectInfo {
            id: "scan".to_string(),
            name: "Scan".to_string(),
            variant: "Scan".to_string(),
        },
    ])
}

#[tauri::command]
#[specta::specta]
pub fn get_effect_schema(
    effect_id: String,
) -> Result<Vec<blade_power::EffectSetting>, String> {
    match effect_id.as_str() {
        "bladepower" => Ok(blade_power::get_schema()),
        "scan" => Ok(scan::get_schema()),
        _ => Err(format!("Schema not found for effect: {}", effect_id)),
    }
}

pub enum EngineCommand {
    StartEffect {
        ip_address: String,
        led_count: u32,
        config: EffectConfig,
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
        settings: EffectConfig,
    },
}

pub struct EngineCommandTx(pub mpsc::Sender<EngineCommand>);

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

    loop {
        let frame_start = Instant::now();
        while let Ok(command) = command_rx.try_recv() {
            match command {
                EngineCommand::StartEffect {
                    ip_address,
                    led_count,
                    config,
                } => {
                    if let Some(removed) = active_effects.remove(&ip_address) {
                        let black_data = vec![0; (removed.led_count * 3) as usize];
                        let destination = format!("{}:4048", ip_address);
                        let _ = ddp::send_ddp_packet(&socket, &destination, 0, &black_data, 0);
                    }

                    let effect: Box<dyn Effect> = match config {
                        EffectConfig::BladePower(c) => Box::new(blade_power::BladePower::new(c)),
                        EffectConfig::Scan(c) => Box::new(scan::Scan::new(c)),
                    };

                    let pixel_count = led_count as usize;
                    active_effects.insert(
                        ip_address,
                        ActiveEffect {
                            led_count,
                            effect,
                            r_channel: vec![0.0; pixel_count],
                            g_channel: vec![0.0; pixel_count],
                            b_channel: vec![0.0; pixel_count],
                        },
                    );
                }
                EngineCommand::UpdateSettings {
                    ip_address,
                    settings,
                } => {
                    if let Some(active_effect) = active_effects.get_mut(&ip_address) {
                        let config_value = match settings {
                            EffectConfig::BladePower(c) => serde_json::to_value(c).unwrap(),
                            EffectConfig::Scan(c) => serde_json::to_value(c).unwrap(),
                        };
                        active_effect.effect.update_config(config_value);
                    }
                }
                EngineCommand::StopEffect { ip_address } => {
                    if let Some(removed) = active_effects.remove(&ip_address) {
                        let black_data = vec![0; (removed.led_count * 3) as usize];
                        let destination = format!("{}:4048", ip_address);
                        let _ = ddp::send_ddp_packet(&socket, &destination, 0, &black_data, 0);
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
            }
        }

        let latest_audio_data = audio_data.inner().0.lock().unwrap().clone();
        frame_count = frame_count.wrapping_add(1);

        for (ip, active_effect) in &mut active_effects {
            let mut frame = vec![0u8; (active_effect.led_count * 3) as usize];
            let pixel_count = frame.len() / 3;

            let mut pure_render = vec![0u8; pixel_count * 3];
            active_effect
                .effect
                .render(&latest_audio_data, &mut pure_render);

            let base_config = active_effect.effect.get_base_config();

            for i in 0..pixel_count {
                active_effect.r_channel[i] = pure_render[i * 3] as f32;
                active_effect.g_channel[i] = pure_render[i * 3 + 1] as f32;
                active_effect.b_channel[i] = pure_render[i * 3 + 2] as f32;
            }

            if base_config.blur > 0.0 {
                dsp::gaussian_blur_1d(&mut active_effect.r_channel, base_config.blur);
                dsp::gaussian_blur_1d(&mut active_effect.g_channel, base_config.blur);
                dsp::gaussian_blur_1d(&mut active_effect.b_channel, base_config.blur);
            }

            if base_config.mirror {
                let half_len = pixel_count / 2;
                let r_clone = active_effect.r_channel.clone();
                let g_clone = active_effect.g_channel.clone();
                let b_clone = active_effect.b_channel.clone();

                if base_config.flip {
                    let first_half_r = &r_clone[0..half_len];
                    let first_half_g = &g_clone[0..half_len];
                    let first_half_b = &b_clone[0..half_len];

                    active_effect.r_channel[0..half_len].copy_from_slice(
                        &first_half_r.iter().rev().cloned().collect::<Vec<f32>>(),
                    );
                    active_effect.g_channel[0..half_len].copy_from_slice(
                        &first_half_g.iter().rev().cloned().collect::<Vec<f32>>(),
                    );
                    active_effect.b_channel[0..half_len].copy_from_slice(
                        &first_half_b.iter().rev().cloned().collect::<Vec<f32>>(),
                    );

                    active_effect.r_channel[pixel_count - half_len..]
                        .copy_from_slice(first_half_r);
                    active_effect.g_channel[pixel_count - half_len..]
                        .copy_from_slice(first_half_g);
                    active_effect.b_channel[pixel_count - half_len..]
                        .copy_from_slice(first_half_b);
                } else {
                    for i in 0..half_len {
                        let mirror_i = pixel_count - 1 - i;
                        active_effect.r_channel[mirror_i] = r_clone[i];
                        active_effect.g_channel[mirror_i] = g_clone[i];
                        active_effect.b_channel[mirror_i] = b_clone[i];
                    }
                }
            } else if base_config.flip {
                active_effect.r_channel.reverse();
                active_effect.g_channel.reverse();
                active_effect.b_channel.reverse();
            }

            let bg_color =
                colors::parse_single_color(&base_config.background_color).unwrap_or([0, 0, 0]);
            for i in 0..pixel_count {
                frame[i * 3] = (active_effect.r_channel[i] as u8)
                    .saturating_add(bg_color[0]);
                frame[i * 3 + 1] = (active_effect.g_channel[i] as u8)
                    .saturating_add(bg_color[1]);
                frame[i * 3 + 2] = (active_effect.b_channel[i] as u8)
                    .saturating_add(bg_color[2]);
            }

            let destination = format!("{}:4048", ip);
            let _ = ddp::send_ddp_packet(&socket, &destination, 0, &frame, frame_count);
            latest_frames.insert(ip.clone(), frame);
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

// --- START: THE FIX ---
// The compiler was right. We need to access the sender INSIDE the State wrapper.
// This fix is applied to ALL Tauri commands.
#[tauri::command]
#[specta::specta]
pub fn start_effect(
    ip_address: String,
    led_count: u32,
    config: EffectConfig,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx
        .0 // Access the sender inside the tuple struct
        .send(EngineCommand::StartEffect {
            ip_address,
            led_count,
            config,
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn stop_effect(
    ip_address: String,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx
        .0
        .send(EngineCommand::StopEffect { ip_address })
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn subscribe_to_frames(
    ip_address: String,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx
        .0
        .send(EngineCommand::Subscribe { ip_address })
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn unsubscribe_from_frames(
    ip_address: String,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx
        .0
        .send(EngineCommand::Unsubscribe { ip_address })
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn set_target_fps(
    fps: u32,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx
        .0
        .send(EngineCommand::SetTargetFps { fps })
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn update_effect_settings(
    ip_address: String,
    settings: EffectConfig,
    command_tx: State<EngineCommandTx>,
) -> Result<(), String> {
    command_tx
        .0
        .send(EngineCommand::UpdateSettings {
            ip_address,
            settings,
        })
        .map_err(|e| e.to_string())
}
// --- END: THE FIX ---