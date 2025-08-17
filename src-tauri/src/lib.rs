mod wled;
mod effects;

#[tauri::command]
fn ping() -> String {
    "pong from Rust!".to_string()
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            ping,
            wled::discover_wled,
            effects::start_effect, // <-- NEW
            effects::stop_effect   // <-- NEW
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}