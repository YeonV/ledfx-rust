// src-tauri/src/audio/devices.rs

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Device;
use serde::Serialize;
use specta::Type;
use tauri::State;
use std::sync::mpsc;
use super::capture::AudioCommand; // Import from the sibling module

#[derive(Serialize, Clone, Type)]
pub struct AudioDevice {
    pub name: String,
}

#[tauri::command]
#[specta::specta]
pub fn get_audio_devices() -> Result<Vec<AudioDevice>, String> {
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
                    device_list.push(AudioDevice { name: loopback_name });
                }
            }
        }
    }
    Ok(device_list)
}

#[tauri::command]
#[specta::specta]
pub fn set_audio_device(
    device_name: String,
    command_tx: State<mpsc::Sender<AudioCommand>>,
) -> Result<(), String> {
    command_tx.send(AudioCommand::ChangeDevice(device_name)).map_err(|e| e.to_string())?;
    Ok(())
}

// This helper is now internal to the devices module.
pub(super) fn find_device(host: &cpal::Host, name: &str, is_loopback: bool) -> Device {
    if is_loopback {
        if let Some(stripped_name) = name.strip_prefix("System Audio (").and_then(|n| n.strip_suffix(")")) {
            if let Some(d) = host.output_devices().unwrap().find(|d| d.name().unwrap_or_default() == stripped_name) {
                return d;
            }
        }
    }
    if let Some(d) = host.input_devices().unwrap().find(|d| d.name().unwrap_or_default() == name) {
        return d;
    }
    host.default_input_device().expect("no input device available")
}