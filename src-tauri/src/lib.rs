pub mod autotrack;
pub mod backup;
pub mod browser;
pub mod capture;
pub mod commands;
pub mod db;
pub mod export;
pub mod discovery;
pub mod goals;
pub mod limits;
pub mod lock;
pub mod proc;
pub mod score;
pub mod stats;
pub mod tracker;

use std::sync::Mutex;
use rusqlite::Connection;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub limits: Mutex<crate::limits::LimitRuntime>,
    pub browser: Mutex<crate::browser::BrowserRuntime>,
    pub site_limits: Mutex<crate::limits::LimitRuntime>,
}

/// Stable data dir: `<data_dir>/Korio/korio.db`. Always AppData so updates
/// (new builds in any folder) keep the user's data.
fn resolve_db_path(data_dir: &std::path::Path) -> std::path::PathBuf {
    data_dir.join("Korio").join("korio.db")
}

/// One-time import: if the AppData DB is absent but a legacy DB sits next to the
/// exe, copy it in. Best-effort; never overwrites an existing AppData DB.
fn migrate_legacy_db(legacy: &std::path::Path, target: &std::path::Path) {
    if !target.exists() && legacy.exists() {
        let _ = std::fs::copy(legacy, target);
    }
}

/// Resolve the DB path (creating the Korio dir) and migrate a legacy next-to-exe DB once.
pub fn db_path() -> std::path::PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let target = resolve_db_path(&base);
    if let Some(dir) = target.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            migrate_legacy_db(&exe_dir.join("korio.db"), &target);
        }
    }
    target
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let conn = crate::db::open(db_path().to_str().expect("db path is valid UTF-8"))
        .expect("failed to open database");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        crate::capture::toggle_capture(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None::<Vec<&str>>,
        ))
        .manage(AppState {
            db: Mutex::new(conn),
            limits: Mutex::new(crate::limits::LimitRuntime::new()),
            browser: Mutex::new(crate::browser::BrowserRuntime::default()),
            site_limits: Mutex::new(crate::limits::LimitRuntime::new()),
        })
        .invoke_handler(tauri::generate_handler![
            crate::commands::list_apps,
            crate::commands::add_app,
            crate::commands::remove_app,
            crate::commands::launch_app,
            crate::commands::running_apps,
            crate::commands::usage_today,
            crate::commands::score_today,
            crate::commands::usage_range,
            crate::commands::daily_totals,
            crate::commands::day_sessions,
            crate::commands::set_app_limit,
            crate::commands::snooze_limit,
            crate::commands::ignore_limit,
            crate::commands::force_close,
            crate::commands::get_settings,
            crate::commands::set_setting,
            crate::commands::export_data,
            crate::commands::backup_db,
            crate::commands::restore_db,
            crate::commands::has_pin,
            crate::commands::set_pin,
            crate::commands::verify_pin,
            crate::commands::clear_pin,
            crate::commands::list_tasks,
            crate::commands::add_task,
            crate::commands::set_task_done,
            crate::commands::update_task_title,
            crate::commands::delete_task,
            crate::commands::clear_done_tasks,
            crate::commands::list_notes,
            crate::commands::add_note,
            crate::commands::update_note,
            crate::commands::delete_note,
            crate::commands::list_goals,
            crate::commands::add_goal,
            crate::commands::update_goal,
            crate::commands::delete_goal,
            crate::commands::goals_progress,
            crate::capture::hide_capture,
            crate::capture::set_capture_shortcut,
            crate::browser::browser_status,
            crate::browser::set_browser_enabled,
            crate::commands::site_usage_today,
            crate::commands::site_usage_range,
            crate::commands::clear_site,
            crate::commands::list_site_caps,
            crate::commands::set_site_limit,
            crate::commands::snooze_site_limit,
            crate::commands::ignore_site_limit,
            crate::commands::block_site,
            crate::commands::list_categories,
            crate::commands::add_category,
            crate::commands::update_category,
            crate::commands::delete_category,
            crate::commands::set_app_category,
            crate::commands::usage_by_category,
            crate::commands::set_note_size,
            crate::commands::clear_alert_topmost,
            crate::commands::site_slices,
            crate::commands::rename_app,
            crate::commands::set_site_name,
            crate::commands::set_site_category,
            crate::commands::list_reminders,
            crate::commands::add_reminder,
            crate::commands::update_reminder,
            crate::commands::set_reminder_done,
            crate::commands::snooze_reminder,
            crate::commands::mark_reminder_fired,
            crate::commands::delete_reminder,
        ])
        .setup(|app| {
            build_tray(app.handle())?;
            setup_main_window(app)?;
            if read_bool_setting(app.handle(), "capture_enabled", true) {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                let _ = app.handle().global_shortcut().register(crate::capture::SHORTCUT);
            }
            crate::tracker::spawn(app.handle().clone());
            if read_bool_setting(app.handle(), "browser_enabled", false) {
                crate::browser::start(app.handle().clone());
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_main(app: &tauri::AppHandle) {
    use tauri::Manager;
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// Bring the main window over everything (used for limit alerts).
pub fn surface_main(app: &tauri::AppHandle) {
    use tauri::Manager;
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_always_on_top(true);
        let _ = w.set_focus();
    }
}

fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    let open_i = MenuItem::with_id(app, "open", "Open Korio", true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit Korio", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_i, &quit_i])?;

    let _tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("Korio")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}

fn setup_main_window(app: &tauri::App) -> tauri::Result<()> {
    use tauri::Manager;
    let win = app.get_webview_window("main").expect("main window exists");

    if !read_bool_setting(app.handle(), "start_minimized", false) {
        let _ = win.show();
        let _ = win.set_focus();
    }

    let handle = app.handle().clone();
    let win_for_event = win.clone();
    win.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            if read_bool_setting(&handle, "close_to_tray", true) {
                api.prevent_close();
                let _ = win_for_event.hide();
            }
        }
    });
    Ok(())
}

fn read_bool_setting(app: &tauri::AppHandle, key: &str, default: bool) -> bool {
    use tauri::Manager;
    let state = app.state::<AppState>();
    let conn = match state.db.lock() {
        Ok(c) => c,
        Err(_) => return default,
    };
    match crate::db::queries::get_setting(&conn, key) {
        Ok(Some(v)) => v == "true",
        _ => default,
    }
}

#[cfg(test)]
mod db_path_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn resolve_uses_appdata_korio() {
        let p = resolve_db_path(&PathBuf::from("/data"));
        assert_eq!(p, PathBuf::from("/data").join("Korio").join("korio.db"));
    }

    #[test]
    fn legacy_db_copied_only_when_target_absent() {
        let base = std::env::temp_dir().join(format!("korio_test_{}", std::process::id()));
        let exe_dir = base.join("exe");
        let data_dir = base.join("data");
        std::fs::create_dir_all(&exe_dir).unwrap();
        std::fs::create_dir_all(&data_dir).unwrap();
        let legacy = exe_dir.join("korio.db");
        std::fs::write(&legacy, b"LEGACY").unwrap();
        let target = data_dir.join("Korio").join("korio.db");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();

        // target absent → copy happens
        migrate_legacy_db(&legacy, &target);
        assert_eq!(std::fs::read(&target).unwrap(), b"LEGACY");

        // target present → no overwrite
        std::fs::write(&target, b"CURRENT").unwrap();
        migrate_legacy_db(&legacy, &target);
        assert_eq!(std::fs::read(&target).unwrap(), b"CURRENT");

        let _ = std::fs::remove_dir_all(&base);
    }
}
