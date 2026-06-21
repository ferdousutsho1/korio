use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use crate::AppState;

pub const KINDS: [&str; 4] = ["stopwatch", "timer", "clock", "pomodoro"];

fn valid(kind: &str) -> bool { KINDS.contains(&kind) }
fn wlabel(kind: &str) -> String { format!("widget-{kind}") }

/// Default logical (width, height) for a widget the first time it opens.
pub fn default_size(kind: &str) -> (f64, f64) {
    match kind {
        "clock" => (250.0, 230.0),
        "stopwatch" => (240.0, 150.0),
        _ => (240.0, 180.0), // timer, pomodoro
    }
}

/// Parse "x,y,w,h" (logical) → (x,y,w,h). Returns None if malformed.
pub fn parse_bounds(s: &str) -> Option<(f64, f64, f64, f64)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 4 { return None; }
    let p: Vec<f64> = parts.iter().filter_map(|x| x.trim().parse().ok()).collect();
    if p.len() == 4 { Some((p[0], p[1], p[2], p[3])) } else { None }
}

#[tauri::command]
pub fn open_widget(app: AppHandle, kind: String) -> Result<(), String> {
    if !valid(&kind) { return Err(format!("unknown widget: {kind}")); }
    let label = wlabel(&kind);
    if let Some(w) = app.get_webview_window(&label) {
        let _ = w.show();
        let _ = w.set_focus();
        return Ok(());
    }
    let (bounds, aot) = {
        let state = app.state::<AppState>();
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        let b = crate::db::queries::get_setting(&conn, &format!("widget.{kind}.bounds")).ok().flatten();
        let a = crate::db::queries::get_setting(&conn, &format!("widget.{kind}.aot")).ok().flatten();
        (b, a)
    };
    let aot = aot.map(|v| v == "true").unwrap_or(true);
    let (dw, dh) = default_size(&kind);
    let url = WebviewUrl::App(format!("index.html?widget={kind}").into());
    let mut builder = WebviewWindowBuilder::new(&app, &label, url)
        .title(format!("Korio · {kind}"))
        .decorations(false)
        .transparent(true)
        .always_on_top(aot)
        .skip_taskbar(true)
        .resizable(true)
        .inner_size(dw, dh)
        .min_inner_size(180.0, 110.0);
    if let Some((x, y, w, h)) = bounds.as_deref().and_then(parse_bounds) {
        builder = builder.inner_size(w, h).position(x, y);
    }
    builder.build().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn close_widget(app: AppHandle, kind: String) -> Result<(), String> {
    if !valid(&kind) { return Err(format!("unknown widget: {kind}")); }
    if let Some(w) = app.get_webview_window(&wlabel(&kind)) {
        w.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn set_widget_always_on_top(app: AppHandle, kind: String, on: bool) -> Result<(), String> {
    if !valid(&kind) { return Err(format!("unknown widget: {kind}")); }
    if let Some(w) = app.get_webview_window(&wlabel(&kind)) {
        w.set_always_on_top(on).map_err(|e| e.to_string())?;
    }
    let state = app.state::<AppState>();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::db::queries::set_setting(&conn, &format!("widget.{kind}.aot"), if on { "true" } else { "false" })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_widget_bounds(app: AppHandle, kind: String, x: f64, y: f64, w: f64, h: f64) -> Result<(), String> {
    if !valid(&kind) { return Err(format!("unknown widget: {kind}")); }
    let state = app.state::<AppState>();
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::db::queries::set_setting(&conn, &format!("widget.{kind}.bounds"), &format!("{x},{y},{w},{h}"))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bounds_valid_and_invalid() {
        assert_eq!(parse_bounds("10,20,240,150"), Some((10.0, 20.0, 240.0, 150.0)));
        assert_eq!(parse_bounds(" 10 , 20 , 240 , 150 "), Some((10.0, 20.0, 240.0, 150.0)));
        assert_eq!(parse_bounds("10,20,240"), None);
        assert_eq!(parse_bounds("bad"), None);
        assert_eq!(parse_bounds(""), None);
        // more than 4 fields, or a non-numeric field, is rejected (no silent token-dropping)
        assert_eq!(parse_bounds("10,20,240,150,99"), None);
        assert_eq!(parse_bounds("10,bad,240,150"), None);
    }

    #[test]
    fn default_size_per_kind_and_validity() {
        assert_eq!(default_size("clock"), (250.0, 230.0));
        assert_eq!(default_size("stopwatch"), (240.0, 150.0));
        assert_eq!(default_size("timer"), (240.0, 180.0));
        assert!(valid("pomodoro"));
        assert!(!valid("nope"));
    }
}
