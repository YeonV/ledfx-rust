// src-tauri/src/effects.rs

use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{mpsc, Mutex, mpsc::TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use lazy_static::lazy_static;
use tauri::{AppHandle, Emitter};
use serde::Serialize; // Import Serialize for the payload struct

// --- 1. The Effect Trait (The Contract) ---
trait Effect: Send {
    fn render_frame(&mut self, leds: u32) -> Vec<u8>;
}

// --- 2. The Rainbow Effect ---
struct RainbowEffect {
    hue: f32,
}
impl Effect for RainbowEffect {
    fn render_frame(&mut self, leds: u32) -> Vec<u8> {
        self.hue = (self.hue + 1.0) % 360.0;
        let rgb = hsv_to_rgb(self.hue, 1.0, 1.0);
        let mut data = Vec::with_capacity((leds * 3) as usize);
        for _ in 0..leds {
            data.extend_from_slice(&rgb);
        }
        data
    }
}

// --- 3. The Scan Effect ---
struct ScanEffect {
    position: u32,
    color: [u8; 3],
}
impl Effect for ScanEffect {
    fn render_frame(&mut self, leds: u32) -> Vec<u8> {
        self.position = (self.position + 1) % leds;
        let mut data = vec![0; (leds * 3) as usize];
        let start_index = (self.position * 3) as usize;
        if start_index + 3 <= data.len() {
            data[start_index..start_index + 3].copy_from_slice(&self.color);
        }
        data
    }
}

// --- 4. The Scroll Effect ---
struct ScrollEffect {
    hue: f32,
}
impl Effect for ScrollEffect {
    fn render_frame(&mut self, leds: u32) -> Vec<u8> {
        self.hue = (self.hue + 0.5) % 360.0;
        let mut data = Vec::with_capacity((leds * 3) as usize);
        for i in 0..leds {
            let pixel_hue = (self.hue + (i as f32 * 10.0)) % 360.0;
            let rgb = hsv_to_rgb(pixel_hue, 1.0, 1.0);
            data.extend_from_slice(&rgb);
        }
        data
    }
}

// --- State Management ---
struct ActiveEffect { shutdown_tx: mpsc::Sender<()>, }
lazy_static! { static ref ACTIVE_EFFECTS: Mutex<HashMap<String, ActiveEffect>> = Mutex::new(HashMap::new()); }

// --- Event Payload Struct ---
#[derive(Serialize, Clone)]
struct EffectFramePayload<'a> {
    ip_address: &'a str,
    pixels: &'a [u8],
}

// --- The Generic Effect Runner ---
fn run_effect(
    ip_address: String,
    led_count: u32,
    mut effect: Box<dyn Effect>,
    shutdown_rx: mpsc::Receiver<()>,
    app_handle: AppHandle,
) {
    let destination = format!("{}:4048", ip_address);
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => { eprintln!("Failed to create UDP socket: {}", e); return; }
    };
    let mut frame_count: u8 = 0;
    let mut last_frontend_update = Instant::now();

    loop {
        if shutdown_rx.try_recv() != Err(TryRecvError::Empty) {
            let black_data = vec![0; (led_count * 3) as usize];
            let _ = send_ddp_packet(&socket, &destination, 0, &black_data, 0);
            println!("Stopping effect for {}", ip_address);
            break;
        }

        frame_count = frame_count.wrapping_add(1);
        let data = effect.render_frame(led_count);

        if let Err(e) = send_ddp_packet(&socket, &destination, 0, &data, frame_count) {
            eprintln!("Failed to send DDP data: {}", e);
            break;
        }

        if last_frontend_update.elapsed() >= Duration::from_millis(100) {
            let payload = EffectFramePayload {
                ip_address: &ip_address,
                pixels: &data,
            };
            app_handle.emit("effect-frame-update", payload).unwrap();
            last_frontend_update = Instant::now();
        }

        thread::sleep(Duration::from_millis(20));
    }
}

// --- NEW: Start and Stop Commands ---
#[tauri::command]
pub async fn start_effect(ip_address: String, led_count: u32, effect_id: String, app_handle: AppHandle) -> Result<(), String> {
    let mut effects = ACTIVE_EFFECTS.lock().unwrap();

    if let Some(_existing) = effects.remove(&ip_address) {
        println!("Stopping existing effect for {} before starting new one.", ip_address);
    }

    let effect: Box<dyn Effect> = match effect_id.as_str() {
        "scan" => Box::new(ScanEffect { position: 0, color: [255, 0, 0] }),
        "scroll" => Box::new(ScrollEffect { hue: 0.0 }),
        "rainbow" | _ => Box::new(RainbowEffect { hue: 0.0 }),
    };

    let (shutdown_tx, shutdown_rx) = mpsc::channel();
    effects.insert(ip_address.clone(), ActiveEffect { shutdown_tx });

    thread::spawn(move || {
        run_effect(ip_address, led_count, effect, shutdown_rx, app_handle);
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_effect(ip_address: String) -> Result<(), String> {
    let mut effects = ACTIVE_EFFECTS.lock().unwrap();
    if let Some(_effect) = effects.remove(&ip_address) {
        println!("Stopping effect for {} via command.", ip_address);
    }
    Ok(())
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
    else if (120.0..180.0).contains(&h) { (0.0, c, x) }
    else if (180.0..240.0).contains(&h) { (0.0, x, c) }
    else if (240.0..300.0).contains(&h) { (x, 0.0, c) }
    else { (c, 0.0, x) };
    [((r_prime + m) * 255.0) as u8, ((g_prime + m) * 255.0) as u8, ((b_prime + m) * 255.0) as u8]
}