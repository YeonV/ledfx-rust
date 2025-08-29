use super::state::ActiveVirtual;
use crate::audio::SharedAudioData;
use crate::types::Device;
use crate::utils::{colors, ddp, dsp};
use std::collections::HashMap;
use std::net::UdpSocket;
use tauri::{AppHandle, Emitter, State};

pub fn render_frame(
    virtuals: &mut HashMap<String, ActiveVirtual>,
    audio_data: &State<SharedAudioData>,
    devices: &HashMap<String, Device>,
    socket: &UdpSocket,
    frame_count: u8,
    app_handle: &AppHandle,
) {
    let latest_audio_data = audio_data.inner().0.lock().unwrap().clone();
    let mut device_buffers: HashMap<String, Vec<u8>> = HashMap::new();
    let mut preview_frames: HashMap<String, Vec<u8>> = HashMap::new();

    for (virtual_id, active_virtual) in virtuals {
        if let Some(effect) = &mut active_virtual.effect {
            let mut virtual_frame = vec![0u8; active_virtual.pixel_count * 3];
            effect.render(&latest_audio_data, &mut virtual_frame);
            let base_config = effect.get_base_config();
            let pixel_count = active_virtual.pixel_count;

            for i in 0..pixel_count {
                active_virtual.r_channel[i] = virtual_frame[i * 3] as f32;
                active_virtual.g_channel[i] = virtual_frame[i * 3 + 1] as f32;
                active_virtual.b_channel[i] = virtual_frame[i * 3 + 2] as f32;
            }

            if base_config.blur > 0.0 {
                dsp::gaussian_blur_1d(&mut active_virtual.r_channel, base_config.blur);
                dsp::gaussian_blur_1d(&mut active_virtual.g_channel, base_config.blur);
                dsp::gaussian_blur_1d(&mut active_virtual.b_channel, base_config.blur);
            }

            if base_config.mirror {
                let half_len = pixel_count / 2;
                let r_clone = active_virtual.r_channel.clone();
                let g_clone = active_virtual.g_channel.clone();
                let b_clone = active_virtual.b_channel.clone();
                if base_config.flip {
                    let first_half_r = &r_clone[0..half_len];
                    let first_half_g = &g_clone[0..half_len];
                    let first_half_b = &b_clone[0..half_len];
                    active_virtual.r_channel[0..half_len].copy_from_slice(&first_half_r.iter().rev().cloned().collect::<Vec<f32>>());
                    active_virtual.g_channel[0..half_len].copy_from_slice(&first_half_g.iter().rev().cloned().collect::<Vec<f32>>());
                    active_virtual.b_channel[0..half_len].copy_from_slice(&first_half_b.iter().rev().cloned().collect::<Vec<f32>>());
                    active_virtual.r_channel[pixel_count - half_len..].copy_from_slice(first_half_r);
                    active_virtual.g_channel[pixel_count - half_len..].copy_from_slice(first_half_g);
                    active_virtual.b_channel[pixel_count - half_len..].copy_from_slice(first_half_b);
                } else {
                    for i in 0..half_len {
                        let mirror_i = pixel_count - 1 - i;
                        active_virtual.r_channel[mirror_i] = r_clone[i];
                        active_virtual.g_channel[mirror_i] = g_clone[i];
                        active_virtual.b_channel[mirror_i] = b_clone[i];
                    }
                }
            } else if base_config.flip {
                active_virtual.r_channel.reverse();
                active_virtual.g_channel.reverse();
                active_virtual.b_channel.reverse();
            }

            let bg_color = colors::parse_single_color(&base_config.background_color).unwrap_or([0, 0, 0]);
            for i in 0..pixel_count {
                virtual_frame[i * 3] = (active_virtual.r_channel[i] as u8).saturating_add(bg_color[0]);
                virtual_frame[i * 3 + 1] = (active_virtual.g_channel[i] as u8).saturating_add(bg_color[1]);
                virtual_frame[i * 3 + 2] = (active_virtual.b_channel[i] as u8).saturating_add(bg_color[2]);
            }

            let mut linear_index = 0;
            for row in &active_virtual.config.matrix_data {
                for cell in row {
                    if let Some(cell_data) = cell {
                        if let Some(device) = devices.get(&cell_data.device_id) {
                            let device_buffer = device_buffers.entry(cell_data.device_id.clone()).or_insert_with(|| vec![0; device.led_count as usize * 3]);
                            let source_idx = linear_index * 3;
                            let dest_idx = cell_data.pixel as usize * 3;
                            if dest_idx + 2 < device_buffer.len() && source_idx + 2 < virtual_frame.len() {
                                device_buffer[dest_idx..dest_idx + 3].copy_from_slice(&virtual_frame[source_idx..source_idx + 3]);
                            }
                        }
                        linear_index += 1;
                    }
                }
            }
            preview_frames.insert(virtual_id.clone(), virtual_frame);
        }
    }

    for (ip, buffer) in &device_buffers {
        let destination = format!("{}:4048", ip);
        let _ = ddp::send_ddp_packet(socket, &destination, 0, buffer, frame_count);
    }

    let preview_payload: HashMap<String, Vec<u8>> = preview_frames.into_iter().collect();
    if !preview_payload.is_empty() {
        app_handle.emit("engine-tick", &preview_payload).unwrap();
    }
}