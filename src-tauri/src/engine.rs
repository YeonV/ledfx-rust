// src-tauri/src/engine.rs

use crate::effects::{Effect, RainbowEffect, ScanEffect, ScrollEffect}; // Import effects from the effects module
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::State;

// --- The Shared State for the Frontend ---
#[derive(Default)]
pub struct SharedFrameBuffer(pub Mutex<HashMap<String, Vec<u8>>>);

// --- The Engine's Internal State ---
struct ActiveEffect {
    led_count: u32,
    effect: Box<dyn Effect>,
}

// --- Engine Commands (Message Passing) ---
pub enum EngineCommand {
    StartEffect {
        ip_address: String,
        led_count: u32,
        effect_id: String,
    },
    StopEffect {
        ip_address: String,
    },
}

// --- The Engine's Main Loop ---
pub fn run_effect_engine(
    command_rx: mpsc::Receiver<EngineCommand>,
    frame_buffer: State<SharedFrameBuffer>,
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

        frame_count = frame_count.wrapping_add(1);
        let mut latest_frames = frame_buffer.0.lock().unwrap();
        for (ip, active_effect) in &mut active_effects {
            let data = active_effect.effect.render_frame(active_effect.led_count);
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
    ip_address: String,
    led_count: u32,
    effect_id: String,
    command_tx: State<mpsc::Sender<EngineCommand>>,
) -> Result<(), String> {
    command_tx.send(EngineCommand::StartEffect { ip_address, led_count, effect_id }).unwrap();
    Ok(())
}

#[tauri::command]
pub fn stop_effect(
    ip_address: String,
    command_tx: State<mpsc::Sender<EngineCommand>>,
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

// --- Helper Functions ---
fn send_ddp_packet(
    socket: &UdpSocket,
    destination: &str,
    offset: u32,
    data: &[u8],
    frame_count: u8,
) -> Result<(), std::io::Error> {
    const MAX_DATA_LEN: usize = 480 * 3;
    let sequence = (frame_count % 15) + 1;
    let mut data_offset = 0;
    while data_offset < data.len() {
        let chunk_end = (data_offset + MAX_DATA_LEN).min(data.len());
        let chunk = &data[data_offset..chunk_end];
        let is_last_packet = chunk_end == data.len();
        let mut header = [0u8; 10];
        header[0] = 0x40 | if is_last_packet { 0x01 } else { 0x00 };
        header[1] = sequence;
        header[2] = 0x01;
        header[3] = 0x01;
        let total_offset = (offset as usize + data_offset) as u32;
        header[4..8].copy_from_slice(&total_offset.to_be_bytes());
        header[8..10].copy_from_slice(&(chunk.len() as u16).to_be_bytes());
        let packet = [&header[..], chunk].concat();
        socket.send_to(&packet, destination)?;
        data_offset += MAX_DATA_LEN;
    }
    Ok(())
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r_prime, g_prime, b_prime) = if (0.0..60.0).contains(&h) { (c, x, 0.0) }
    else if (60.0..120.0).contains(&h) { (x, c, 0.0) }
    else if (180.0..240.0).contains(&h) { (0.0, x, c) }
    else if (120.0..180.0).contains(&h) { (0.0, c, x) }
    else if (240.0..300.0).contains(&h) { (x, 0.0, c) }
    else { (c, 0.0, x) };
    [((r_prime + m) * 255.0) as u8, ((g_prime + m) * 255.0) as u8, ((b_prime + m) * 255.0) as u8]
}