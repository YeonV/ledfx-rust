use super::{AudioAnalysisData, AudioCommand, DspSettings};
use crate::audio::shared_processing::build_and_play_stream_shared;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream};
use std::sync::{mpsc, Arc, Mutex};

pub fn get_desktop_devices_impl() -> Result<super::AudioDevicesInfo, String> {
    let host = cpal::default_host();
    let mut input_devices: Vec<super::AudioDevice> = Vec::new();
    #[cfg_attr(not(target_os = "windows"), allow(unused_mut))]
    let mut loopback_devices: Vec<super::AudioDevice> = Vec::new();

    if let Ok(devices) = host.input_devices() {
        for device in devices {
            if let Ok(name) = device.name() {
                input_devices.push(super::AudioDevice { name });
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(devices) = host.output_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    let loopback_name = format!("System Audio ({})", name);
                    loopback_devices.push(super::AudioDevice {
                        name: loopback_name,
                    });
                }
            }
        }
    }

    let mut default_device_name: Option<String> = None;
    if let Some(default_output) = host.default_output_device() {
        if let Ok(target_name) = default_output.name() {
            let target_loopback_name = format!("System Audio ({})", target_name);
            if let Some(device) = loopback_devices
                .iter()
                .find(|d| d.name == target_loopback_name)
            {
                default_device_name = Some(device.name.clone());
            }
        }
    }

    if default_device_name.is_none() && !loopback_devices.is_empty() {
        default_device_name = Some(loopback_devices[0].name.clone());
    }

    if default_device_name.is_none() {
        if let Some(default_input) = host.default_input_device() {
            if let Ok(name) = default_input.name() {
                if input_devices.iter().any(|d| d.name == name) {
                    default_device_name = Some(name);
                }
            }
        }
    }

    if default_device_name.is_none() && !input_devices.is_empty() {
        default_device_name = Some(input_devices[0].name.clone());
    }

    let mut all_devices = loopback_devices;
    all_devices.extend(input_devices);

    Ok(super::AudioDevicesInfo {
        devices: all_devices,
        default_device_name,
    })
}

pub fn set_desktop_device_impl(
    device_name: String,
    command_tx: tauri::State<mpsc::Sender<AudioCommand>>,
) -> Result<(), String> {
    command_tx
        .send(AudioCommand::ChangeDevice(device_name))
        .map_err(|e| e.to_string())
}

pub fn run_desktop_capture(
    command_rx: mpsc::Receiver<AudioCommand>,
    audio_data: Arc<Mutex<AudioAnalysisData>>,
    dsp_settings: Arc<Mutex<DspSettings>>,
) {
    let host = cpal::default_host();
    let mut current_stream: Option<Stream> = None;
    let mut current_device_name: Option<String> = None;

    loop {
        if let Ok(command) = command_rx.try_recv() {
            match command {
                AudioCommand::ChangeDevice(device_name) => {
                    println!(
                        "[AUDIO] Received command to change audio device to: {}",
                        device_name
                    );
                    current_device_name = Some(device_name.clone());

                    if let Some(stream) = current_stream.take() {
                        stream.pause().expect("Failed to pause stream");
                        drop(stream);
                    }

                    let is_loopback =
                        cfg!(target_os = "windows") && device_name.starts_with("System Audio (");

                    if let Some(device) = find_device(&host, &device_name, is_loopback) {
                        let config = if is_loopback {
                            device
                                .default_output_config()
                                .expect("no default output config")
                        } else {
                            device
                                .default_input_config()
                                .expect("no default input config")
                        };
                        let stream = build_and_play_stream_shared(
                            device,
                            config,
                            audio_data.clone(),
                            dsp_settings.clone(),
                        );
                        current_stream = Some(stream);
                    } else {
                        eprintln!("[AUDIO] Could not find requested device: {}", device_name);
                        current_device_name = None;
                    }
                }
                AudioCommand::UpdateSettings(new_settings) => {
                    println!("[AUDIO] Received new DSP settings.");
                    let mut settings = dsp_settings.lock().unwrap();
                    *settings = new_settings;
                }
                AudioCommand::RestartStream => {
                    if let Some(device_name) = current_device_name.clone() {
                        println!(
                            "[AUDIO] Received command to restart audio stream for device: {}",
                            device_name
                        );

                        if let Some(stream) = current_stream.take() {
                            stream.pause().expect("Failed to pause stream");
                            drop(stream);
                        }

                        let is_loopback = cfg!(target_os = "windows")
                            && device_name.starts_with("System Audio (");

                        if let Some(device) = find_device(&host, &device_name, is_loopback) {
                            let config = if is_loopback {
                                device
                                    .default_output_config()
                                    .expect("no default output config")
                            } else {
                                device
                                    .default_input_config()
                                    .expect("no default input config")
                            };
                            let stream = build_and_play_stream_shared(
                                device,
                                config,
                                audio_data.clone(),
                                dsp_settings.clone(),
                            );
                            current_stream = Some(stream);
                        } else {
                            eprintln!(
                                "[AUDIO] Could not find previous device to restart: {}",
                                device_name
                            );
                            current_device_name = None;
                        }
                    } else {
                        println!("[AUDIO] Cannot restart stream, no device is currently active.");
                    }
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn find_device(host: &cpal::Host, name: &str, is_loopback: bool) -> Option<Device> {
    if is_loopback {
        if let Some(stripped_name) = name
            .strip_prefix("System Audio (")
            .and_then(|n| n.strip_suffix(')'))
        {
            if let Ok(mut devices) = host.output_devices() {
                return devices.find(|d| d.name().unwrap_or_default() == stripped_name);
            }
        }
    }
    if let Ok(mut devices) = host.input_devices() {
        return devices.find(|d| d.name().unwrap_or_default() == name);
    }
    None
}
