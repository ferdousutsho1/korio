use crate::db::queries::{self, App, UsageSlice};
use crate::discovery::{self, RunningApp};
use crate::AppState;
use tauri::State;

fn day_bounds_local() -> (i64, i64) {
    use chrono::{Datelike, Local, TimeZone, Timelike};
    let now = Local::now();
    // Local midnight. On DST "spring-forward" days some zones skip 00:00, so
    // `.earliest()` can be None — fall back to (now - seconds since midnight)
    // rather than panicking.
    let from = Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .earliest()
        .map(|dt| dt.timestamp())
        .unwrap_or_else(|| now.timestamp() - now.num_seconds_from_midnight() as i64);
    (from, from + 86_400)
}

#[tauri::command]
pub fn list_apps(state: State<AppState>) -> Result<Vec<App>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_apps(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_app(
    state: State<AppState>,
    display_name: String,
    exe_name: String,
    kind: String,
    color: String,
) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::add_app(&conn, &display_name, &exe_name, &kind, &color).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_app(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::remove_app(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn running_apps() -> Result<Vec<RunningApp>, String> {
    Ok(discovery::running_apps())
}

#[tauri::command]
pub fn usage_today(state: State<AppState>) -> Result<Vec<UsageSlice>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (from, to) = day_bounds_local();
    queries::usage_between(&conn, from, to).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn score_today(state: State<AppState>) -> Result<u8, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (from, to) = day_bounds_local();
    let slices = queries::usage_between(&conn, from, to).map_err(|e| e.to_string())?;
    Ok(crate::score::focus_score(&slices))
}
