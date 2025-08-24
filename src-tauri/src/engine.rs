use crate::audio::SharedAudioData;
use crate::effects::{blade, legacy, simple, Effect};
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

#[derive(Deserialize, Serialize, Type, Clone)]
#[serde(tag = "mode", content = "config")]
pub enum EffectConfig {
    #[serde(rename = "legacy")]
    Legacy(crate::effects::legacy::blade_power::BladePowerLegacyConfig),
    #[serde(rename = "blade")]
    Blade(crate::effects::blade::blade_power::BladePowerConfig),
}

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
                    let effect: Option<Box<dyn Effect>> = if let Some(config_data) = config {
                        match config_data {
                            EffectConfig::Legacy(conf) => {
                                Some(Box::new(legacy::blade_power::BladePowerLegacy::new(conf)))
                            }
                            EffectConfig::Blade(conf) => Some(Box::new(
                                blade::blade_power::BladePower::new(conf),
                            )),
                        }
                    } else {
                        // Use the new constructors for simple effects
                        match effect_id.as_str() {
                            "scan" => Some(Box::new(simple::ScanEffect::new())),
                            "scroll" => Some(Box::new(simple::ScrollEffect::new())),
                            _ => Some(Box::new(simple::RainbowEffect::new())),
                        }
                    };

                    if let Some(effect) = effect {
                        let pixel_count = led_count as usize;
                        active_effects.insert(ip_address, ActiveEffect {
                            led_count,
                            effect,
                            r_channel: vec![0.0; pixel_count],
                            g_channel: vec![0.0; pixel_count],
                            b_channel: vec![0.0; pixel_count],
                        });
                    } else {
                        eprintln!("Failed to create effect with id '{}'", effect_id);
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
                EngineCommand::UpdateSettings {
                    ip_address,
                    settings,
                } => {
                    if let Some(active_effect) = active_effects.get_mut(&ip_address) {
                        if let Some(config_enum) = settings {
                            let config_value = match config_enum {
                                EffectConfig::Legacy(config) => serde_json::to_value(config).unwrap(),
                                EffectConfig::Blade(config) => serde_json::to_value(config).unwrap(),
                            };
                            active_effect.effect.update_config(config_value);
                        }
                    }
                }
            }
        }

        let latest_audio_data = audio_data.inner().0.lock().unwrap().clone();
        frame_count = frame_count.wrapping_add(1);

        for (ip, active_effect) in &mut active_effects {
            let mut frame = vec![0u8; (active_effect.led_count * 3) as usize];
            let pixel_count = frame.len() / 3;
            
            // --- START: THE FINAL, CORRECT PIPELINE ---

            // 1. Render the pure effect into a temporary buffer.
            let mut pure_render = vec![0u8; pixel_count * 3];
            active_effect.effect.render(&latest_audio_data, &mut pure_render);
            
            let base_config = active_effect.effect.get_base_config();
            
            // 2. Deconstruct into float buffers for processing.
            for i in 0..pixel_count {
                active_effect.r_channel[i] = pure_render[i * 3] as f32;
                active_effect.g_channel[i] = pure_render[i * 3 + 1] as f32;
                active_effect.b_channel[i] = pure_render[i * 3 + 2] as f32;
            }

            // 3. Apply Blur.
            if base_config.blur > 0.0 {
                dsp::gaussian_blur_1d(&mut active_effect.r_channel, base_config.blur);
                dsp::gaussian_blur_1d(&mut active_effect.g_channel, base_config.blur);
                dsp::gaussian_blur_1d(&mut active_effect.b_channel, base_config.blur);
            }
            
            // 4. Handle Mirror and Flip combinations.
            if base_config.mirror {
                let half_len = pixel_count / 2;
                let r_clone = active_effect.r_channel.clone();
                let g_clone = active_effect.g_channel.clone();
                let b_clone = active_effect.b_channel.clone();

                if base_config.flip { // Center-out
                    let first_half_r = &r_clone[0..half_len];
                    let first_half_g = &g_clone[0..half_len];
                    let first_half_b = &b_clone[0..half_len];

                    // Left side of strip gets first half of bar, reversed
                    active_effect.r_channel[0..half_len].copy_from_slice(&first_half_r.iter().rev().cloned().collect::<Vec<f32>>());
                    active_effect.g_channel[0..half_len].copy_from_slice(&first_half_g.iter().rev().cloned().collect::<Vec<f32>>());
                    active_effect.b_channel[0..half_len].copy_from_slice(&first_half_b.iter().rev().cloned().collect::<Vec<f32>>());

                    // Right side of strip gets first half of bar
                    active_effect.r_channel[pixel_count - half_len..].copy_from_slice(first_half_r);
                    active_effect.g_channel[pixel_count - half_len..].copy_from_slice(first_half_g);
                    active_effect.b_channel[pixel_count - half_len..].copy_from_slice(first_half_b);
                    
                } else { // Outside-in
                    for i in 0..half_len {
                        let mirror_i = pixel_count - 1 - i;
                        active_effect.r_channel[mirror_i] = r_clone[i];
                        active_effect.g_channel[mirror_i] = g_clone[i];
                        active_effect.b_channel[mirror_i] = b_clone[i];
                    }
                }
            } else if base_config.flip { // Standard flip
                active_effect.r_channel.reverse();
                active_effect.g_channel.reverse();
                active_effect.b_channel.reverse();
            }
            
            // 5. Composite the final frame.
            let bg_color = colors::parse_single_color(&base_config.background_color).unwrap_or([0,0,0]);
            for i in 0..pixel_count {
                frame[i * 3]     = (active_effect.r_channel[i] as u8).saturating_add(bg_color[0]);
                frame[i * 3 + 1] = (active_effect.g_channel[i] as u8).saturating_add(bg_color[1]);
                frame[i * 3 + 2] = (active_effect.b_channel[i] as u8).saturating_add(bg_color[2]);
            }
            // --- END: THE FINAL, CORRECT PIPELINE ---

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

// All Tauri commands are unchanged
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