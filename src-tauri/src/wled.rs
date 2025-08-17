// src-tauri/src/wled.rs

use mdns_sd::{ServiceDaemon, ServiceEvent};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use specta::Type;

#[derive(Deserialize, Clone, Serialize, Type)]
pub struct LedsInfo {
    pub count: u32,
}

#[derive(Deserialize, Clone, Serialize, Type)]
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

#[derive(Serialize, Clone, Type)]
pub struct WledDevice {
    pub ip_address: String,
    pub port: u16,
    pub name: String,
    pub version: String,
    pub leds: LedsInfo,
    pub udp_port: u16,
    pub architecture: String,
    pub maps: Vec<MapInfo>,
}

#[tauri::command]
#[specta::specta]
pub async fn discover_wled(app_handle: AppHandle, duration_secs: Option<u32>) -> Result<(), String> {
    const WLED_SERVICE_TYPE: &str = "_wled._tcp.local.";
    let search_duration = Duration::from_secs(duration_secs.unwrap_or(10) as u64);
    let mdns = ServiceDaemon::new().map_err(|e| e.to_string())?;
    let receiver = mdns.browse(WLED_SERVICE_TYPE).map_err(|e| e.to_string())?;

    tokio::spawn(async move {
        let http_client = reqwest::Client::new();
        let _ = tokio::time::timeout(search_duration, async {
            while let Ok(event) = receiver.recv_async().await {
                if let ServiceEvent::ServiceResolved(info) = event {
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
                            app_handle.emit("wled-device-found", &enriched_device).unwrap();
                        }
                    }
                }
            }
        }).await;
    });

    Ok(())
}