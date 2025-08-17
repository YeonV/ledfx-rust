// src-tauri/src/utils.rs

use std::net::UdpSocket;

// --- DDP Packet Helper ---
pub fn send_ddp_packet(
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
        header[2] = 0x01; // Data Type RGB
        header[3] = 0x01; // Source ID
        let total_offset = (offset as usize + data_offset) as u32;
        header[4..8].copy_from_slice(&total_offset.to_be_bytes());
        header[8..10].copy_from_slice(&(chunk.len() as u16).to_be_bytes());

        let packet = [&header[..], chunk].concat();
        socket.send_to(&packet, destination)?;

        data_offset += MAX_DATA_LEN;
    }

    Ok(())
}

// --- Color Helper ---
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [u8; 3] {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r_prime, g_prime, b_prime) = if (0.0..60.0).contains(&h) { (c, x, 0.0) }
    else if (60.0..120.0).contains(&h) { (x, c, 0.0) }
    else if (120.0..180.0).contains(&h) { (0.0, c, x) }
    else if (180.0..240.0).contains(&h) { (0.0, x, c) }
    else if (240.0..300.0).contains(&h) { (x, 0.0, c) }
    else { (c, 0.0, x) };
    [
        ((r_prime + m) * 255.0) as u8,
        ((g_prime + m) * 255.0) as u8,
        ((b_prime + m) * 255.0) as u8,
    ]
}