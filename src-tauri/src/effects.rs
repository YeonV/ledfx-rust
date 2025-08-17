// src-tauri/src/effects.rs

use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::{mpsc, Mutex, mpsc::TryRecvError};
use std::thread;
use std::time::{Duration, Instant}; // <-- Import Instant for throttling
use lazy_static::lazy_static;
use tauri::{AppHandle, Emitter}; // <-- Import AppHandle and Emitter

// --- NEW: A struct for the event payload ---
#[derive(serde::Serialize, Clone)]
struct EffectFramePayload<'a> {
    ip_address: &'a str,
    pixels: &'a [u8],
}


// --- State Management for Active Effects ---
struct ActiveEffect {
    shutdown_tx: mpsc::Sender<()>,
}

lazy_static! {
    static ref ACTIVE_EFFECTS: Mutex<HashMap<String, ActiveEffect>> = Mutex::new(HashMap::new());
}

// --- The DDP Effect Task ---
fn run_dummy_effect(ip_address: String, shutdown_rx: mpsc::Receiver<()>, app_handle: AppHandle) {
    let destination = format!("{}:4048", ip_address);
    println!("Starting DDP effect for {}", destination);

    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create UDP socket: {}", e);
            return;
        }
    };
    // ... socket and conn creation remains the same ...

    let mut frame_count: u8 = 0;
    let mut hue: f32 = 0.0;
    let mut last_frontend_update = Instant::now(); // For throttling

    loop {
        if shutdown_rx.try_recv() != Err(TryRecvError::Empty) {
            // ... shutdown logic remains the same ...
            break;
        }

        frame_count = frame_count.wrapping_add(1);
        hue = (hue + 1.0) % 360.0;
        let rgb = hsv_to_rgb(hue, 1.0, 1.0);
        
        let mut data = Vec::with_capacity(100 * 3);
        for _ in 0..100 {
            data.extend_from_slice(&rgb);
        }

        // Send to the device (no change here)
        if let Err(e) = send_ddp_packet(&socket, &destination, 0, &data, frame_count) {
            eprintln!("Failed to send DDP data: {}", e);
            break;
        }

        // --- NEW: Send frame to frontend (throttled) ---
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



// --- Manual DDP Packet Sending Function (remains the same) ---
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

// --- The Tauri Command ---
#[tauri::command]
pub async fn toggle_ddp_effect(ip_address: String, app_handle: AppHandle) -> Result<bool, String> {
    let mut effects = ACTIVE_EFFECTS.lock().unwrap();

    if let Some(_effect) = effects.remove(&ip_address) {
        Ok(false) // Return false for "inactive"
    } else {
        let (shutdown_tx, shutdown_rx) = mpsc::channel();
        let effect = ActiveEffect { shutdown_tx };
        effects.insert(ip_address.clone(), effect);

        // Pass the app_handle to the new thread
        thread::spawn(move || {
            run_dummy_effect(ip_address, shutdown_rx, app_handle);
        });

        Ok(true) // Return true for "active"
    }
}

// Helper function to convert HSV to RGB (remains the same)
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