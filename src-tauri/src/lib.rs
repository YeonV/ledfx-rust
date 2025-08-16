// src-tauri/src/lib.rs

// --- NEW: Define your command here, in the library ---
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
        // --- THE FIX: Register ALL your commands here ---
        .invoke_handler(tauri::generate_handler![
            greet,
            ping // Add the new command to the list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}