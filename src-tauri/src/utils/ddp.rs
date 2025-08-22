// src-tauri/src/utils/ddp.rs

use std::net::UdpSocket;

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
