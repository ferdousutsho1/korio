pub mod commands;
pub mod db;
pub mod discovery;
pub mod score;
pub mod stats;
pub mod tracker;

use std::sync::Mutex;
use rusqlite::Connection;

pub struct AppState {
    pub db: Mutex<Connection>,
}

/// Portable mode: korio.db next to the exe if that directory looks writable;
/// otherwise %APPDATA%\Korio\korio.db.
pub fn db_path() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("korio.db");
            if dir.metadata().map(|m| !m.permissions().readonly()).unwrap_or(false) {
                return p;
            }
        }
    }
    let base = dirs::data_dir().unwrap_or(std::path::PathBuf::from("."));
    let dir = base.join("Korio");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("korio.db")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let conn = crate::db::open(db_path().to_str().expect("db path is valid UTF-8"))
        .expect("failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { db: Mutex::new(conn) })
        .invoke_handler(tauri::generate_handler![
            crate::commands::list_apps,
            crate::commands::add_app,
            crate::commands::remove_app,
            crate::commands::running_apps,
            crate::commands::usage_today,
            crate::commands::score_today,
        ])
        .setup(|app| {
            crate::tracker::spawn(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
