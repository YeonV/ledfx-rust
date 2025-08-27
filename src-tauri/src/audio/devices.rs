// src-tauri/src/audio/devices.rs

use super::{AudioCommand, AudioDevice};
use cpal::traits::{DeviceTrait, HostTrait};
use std::sync::mpsc;
use tauri::State;

// This is now JUST the implementation, the tauri::command macros are removed.
pub fn get_desktop_devices_impl() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let mut device_list: Vec<AudioDevice> = Vec::new();
    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                device_list.push(AudioDevice { name });
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(devices) = host.output_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    let loopback_name = format!("System Audio ({})", name);
                    device_list.push(AudioDevice {
                        name: loopback_name,
                    });
                }
            }
        }
    }
    Ok(device_list)
}

// This is now JUST the implementation.
pub fn set_desktop_device_impl(
    device_name: String,
    command_tx: State<mpsc::Sender<AudioCommand>>,
) -> Result<(), String> {
    command_tx
        .send(AudioCommand::ChangeDevice(device_name))
        .map_err(|e| e.to_string())
}
