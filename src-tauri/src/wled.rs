// src-tauri/src/wled.rs

// --- Make necessary items from other crates available in this module ---
use mdns_sd::{ServiceDaemon, ServiceEvent};
use serde::{Deserialize, Serialize}; // We can combine these
use std::collections::HashMap;
use std::time::Duration;
use tauri::{Emitter, AppHandle}; // We need AppHandle for the command

// --- Structs to match the WLED JSON API response ---
// These need to be `pub` so the main WledDevice struct can use them.
#[derive(Deserialize, Clone, Serialize)]
pub struct LedsInfo {
    pub count: u32,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct MapInfo {
    pub id: u32,
}

#[derive(Deserialize, Clone, Serialize)]
struct WledApiResponse {
    name: String,
    ver: String,
    leds: LedsInfo,
    udpport: u16,
    arch: String,
    maps: Vec<MapInfo>,
}

// --- The main WledDevice struct ---
// This must be `pub` so the frontend can receive it.
#[derive(Serialize, Clone)]
pub struct WledDevice {
    ip_address: String,
    port: u16,
    name: String,
    version: String,
    leds: LedsInfo,
    udp_port: u16,
    architecture: String,
    maps: Vec<MapInfo>,
}

// --- The discovery command ---
// This must be `pub` so `lib.rs` can register it.
#[tauri::command]
pub async fn discover_wled(app_handle: AppHandle, duration_secs: Option<u64>) -> Result<(), String> {
    const WLED_SERVICE_TYPE: &str = "_wled._tcp.local.";
    let search_duration = Duration::from_secs(duration_secs.unwrap_or(10));

    let mdns = ServiceDaemon::new().map_err(|e| e.to_string())?;
    let receiver = mdns.browse(WLED_SERVICE_TYPE).map_err(|e| e.to_string())?;

    tokio::spawn(async move {
        println!("BACKGROUND SCAN: Started for {} seconds.", search_duration.as_secs());
        
        let mut found_devices = HashMap::new();
        let http_client = reqwest::Client::new();

        let _ = tokio::time::timeout(search_duration, async {
            while let Ok(event) = receiver.recv_async().await {
                if let ServiceEvent::ServiceResolved(info) = event {
                    let key = info.get_fullname().to_string();
                    if found_devices.contains_key(&key) {
                        continue;
                    }

                    let ip_address = match info.get_addresses().iter().next() {
                        Some(addr) => addr.to_string(),
                        None => continue,
                    };
                    let port = info.get_port();
                    
                    let url = format!("http://{}:{}/json/info", ip_address, port);
                    if let Ok(response) = http_client.get(&url).send().await {
                        if let Ok(api_data) = response.json::<WledApiResponse>().await {
                            let enriched_device = WledDevice {
                                ip_address: ip_address.clone(),
                                port,
                                name: api_data.name,
                                version: api_data.ver,
                                leds: api_data.leds,
                                udp_port: api_data.udpport,
                                architecture: api_data.arch,
                                maps: api_data.maps,
                            };

                            println!("ENRICHED DEVICE: {:?}", enriched_device.name);
                            app_handle.emit("wled-device-found", &enriched_device).unwrap();
                            found_devices.insert(key, ());
                        }
                    }
                }
            }
        }).await;

        println!("BACKGROUND SCAN: Finished.");
    });

    Ok(())
}