use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use crate::AppState;

/// The global hotkey. Windows-only app, so a plain "Ctrl+Alt+K" accelerator string.
pub const SHORTCUT: &str = "Ctrl+Alt+K";

/// Get-or-create the capture window, then show + focus it.
pub fn show_capture(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("capture") {
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "capture", WebviewUrl::App("index.html?capture=1".into()))
        .title("Korio Capture")
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .inner_size(440.0, 150.0)
        .center()
        .build();
}

/// Hotkey behavior: toggle the capture window (hide if visible, else show+focus).
pub fn toggle_capture(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("capture") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
            return;
        }
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    show_capture(app);
}

#[tauri::command]
pub fn hide_capture(app: AppHandle) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("capture") {
        w.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_capture_shortcut(app: AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let gs = app.global_shortcut();
    if enabled {
        let _ = gs.unregister(SHORTCUT); // avoid double-register errors
        gs.register(SHORTCUT).map_err(|e| e.to_string())?;
    } else {
        let _ = gs.unregister(SHORTCUT);
    }
    let state = app.state::<AppState>();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::db::queries::set_setting(&conn, "capture_enabled", if enabled { "true" } else { "false" })
        .map_err(|e| e.to_string())
}
