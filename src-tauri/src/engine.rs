// src-tauri/src/engine.rs

use crate::effects::{Effect, RainbowEffect, ScanEffect, ScrollEffect, BladePowerEffect};
use crate::audio::SharedAudioData;
use crate::utils::send_ddp_packet;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::State;

#[derive(Default)]
pub struct SharedFrameBuffer(pub Mutex<HashMap<String, Vec<u8>>>);

struct ActiveEffect {
    led_count: u32,
    effect: Box<dyn Effect>,
}

pub enum EngineCommand {
    StartEffect { ip_address: String, led_count: u32, effect_id: String },
    StopEffect { ip_address: String },
}

pub fn run_effect_engine(
    command_rx: mpsc::Receiver<EngineCommand>,
    frame_buffer: State<SharedFrameBuffer>,
    audio_data: State<SharedAudioData>,
) {
    let mut active_effects: HashMap<String, ActiveEffect> = HashMap::new();
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    let mut frame_count: u8 = 0;

    loop {
        while let Ok(command) = command_rx.try_recv() {
            match command {
                EngineCommand::StartEffect { ip_address, led_count, effect_id } => {
                    println!("ENGINE: Starting effect '{}' for {}", effect_id, ip_address);
                    let effect: Box<dyn Effect> = match effect_id.as_str() {
                        "scan" => Box::new(ScanEffect { position: 0, color: [255, 0, 0] }),
                        "scroll" => Box::new(ScrollEffect { hue: 0.0 }),
                        "bladepower" => Box::new(BladePowerEffect),
                        _ => Box::new(RainbowEffect { hue: 0.0 }),
                    };
                    active_effects.insert(ip_address, ActiveEffect { led_count, effect });
                }
                EngineCommand::StopEffect { ip_address } => {
                    println!("ENGINE: Stopping effect for {}", ip_address);
                    if active_effects.remove(&ip_address).is_some() {
                        let black_data = vec![0; 300 * 3];
                        let destination = format!("{}:4048", ip_address);
                        let _ = send_ddp_packet(&socket, &destination, 0, &black_data, 0);
                        frame_buffer.0.lock().unwrap().remove(&ip_address);
                    }
                }
            }
        }

        let latest_audio_data = audio_data.0.lock().unwrap().clone();
        
        // --- DIAGNOSTIC LOGGING ---
        // if !active_effects.is_empty() {
        //     println!("ENGINE THREAD: Read Volume = {:.4}", latest_audio_data.volume);
        // }

        frame_count = frame_count.wrapping_add(1);
        let mut latest_frames = frame_buffer.0.lock().unwrap();

        for (ip, active_effect) in &mut active_effects {
            let data = active_effect.effect.render_frame(active_effect.led_count, &latest_audio_data);
            let destination = format!("{}:4048", ip);
            let _ = send_ddp_packet(&socket, &destination, 0, &data, frame_count);
            latest_frames.insert(ip.clone(), data);
        }

        thread::sleep(Duration::from_millis(16));
    }
}

// --- Tauri Commands ---
#[tauri::command]
pub fn start_effect(
    ip_address: String, led_count: u32, effect_id: String,
    command_tx: State<mpsc::Sender<EngineCommand>>,
) -> Result<(), String> {
    command_tx.send(EngineCommand::StartEffect { ip_address, led_count, effect_id }).unwrap();
    Ok(())
}

#[tauri::command]
pub fn stop_effect(
    ip_address: String, command_tx: State<mpsc::Sender<EngineCommand>>,
) -> Result<(), String> {
    command_tx.send(EngineCommand::StopEffect { ip_address }).unwrap();
    Ok(())
}

#[tauri::command]
pub fn get_latest_frames(
    frame_buffer: State<SharedFrameBuffer>,
) -> Result<HashMap<String, Vec<u8>>, String> {
    let frames = frame_buffer.0.lock().unwrap().clone();
    Ok(frames)
}