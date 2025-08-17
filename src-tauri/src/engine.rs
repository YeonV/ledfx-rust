// src-tauri/src/engine.rs

use crate::effects::{Effect, RainbowEffect, ScanEffect, ScrollEffect, BladePowerEffect};
use crate::audio::SharedAudioData;
use crate::utils::send_ddp_packet;
use std::collections::{HashMap, HashSet};
use std::net::UdpSocket;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{State, AppHandle, Emitter};

struct ActiveEffect {
    led_count: u32,
    effect: Box<dyn Effect>,
}

pub enum EngineCommand {
    StartEffect { ip_address: String, led_count: u32, effect_id: String },
    StopEffect { ip_address: String },
    Subscribe { ip_address: String },
    Unsubscribe { ip_address: String },
    SetTargetFps { fps: u32 },
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
                EngineCommand::StartEffect { ip_address, led_count, effect_id } => {
                    let effect: Box<dyn Effect> = match effect_id.as_str() {
                        "scan" => Box::new(ScanEffect { position: 0, color: [255, 0, 0] }),
                        "scroll" => Box::new(ScrollEffect { hue: 0.0 }),
                        "bladepower" => Box::new(BladePowerEffect),
                        _ => Box::new(RainbowEffect { hue: 0.0 }),
                    };
                    active_effects.insert(ip_address, ActiveEffect { led_count, effect });
                }
                EngineCommand::StopEffect { ip_address } => {
                    if active_effects.remove(&ip_address).is_some() {
                        let black_data = vec![0; 300 * 3];
                        let destination = format!("{}:4048", ip_address);
                        let _ = send_ddp_packet(&socket, &destination, 0, &black_data, 0);
                        latest_frames.remove(&ip_address);
                    }
                }
                EngineCommand::Subscribe { ip_address } => { subscribed_ips.insert(ip_address); }
                EngineCommand::Unsubscribe { ip_address } => { subscribed_ips.remove(&ip_address); }
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
            let data = active_effect.effect.render_frame(active_effect.led_count, &latest_audio_data);
            let destination = format!("{}:4048", ip);
            let _ = send_ddp_packet(&socket, &destination, 0, &data, frame_count);
            latest_frames.insert(ip.clone(), data);
        }

        let payload: HashMap<String, Vec<u8>> = latest_frames.iter()
            .filter(|(ip, _)| subscribed_ips.contains(*ip))
            .map(|(ip, data)| (ip.clone(), data.clone()))
            .collect();

        if !payload.is_empty() { app_handle.emit("engine-tick", &payload).unwrap(); }

        let frame_duration = frame_start.elapsed();
        if let Some(sleep_duration) = target_frame_duration.checked_sub(frame_duration) {
            thread::sleep(sleep_duration);
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn start_effect(ip_address: String, led_count: u32, effect_id: String, command_tx: State<mpsc::Sender<EngineCommand>>) -> Result<(), String> {
    command_tx.send(EngineCommand::StartEffect { ip_address, led_count, effect_id }).unwrap();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn stop_effect(ip_address: String, command_tx: State<mpsc::Sender<EngineCommand>>) -> Result<(), String> {
    command_tx.send(EngineCommand::StopEffect { ip_address }).unwrap();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn subscribe_to_frames(ip_address: String, command_tx: State<mpsc::Sender<EngineCommand>>) -> Result<(), String> {
    command_tx.send(EngineCommand::Subscribe { ip_address }).unwrap();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn unsubscribe_from_frames(ip_address: String, command_tx: State<mpsc::Sender<EngineCommand>>) -> Result<(), String> {
    command_tx.send(EngineCommand::Unsubscribe { ip_address }).unwrap();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_target_fps(fps: u32, command_tx: State<mpsc::Sender<EngineCommand>>) -> Result<(), String> {
    command_tx.send(EngineCommand::SetTargetFps { fps }).unwrap();
    Ok(())
}