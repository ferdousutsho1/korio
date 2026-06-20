use crate::db::queries::{self, App, SessionRow, UsageSlice};
use crate::discovery::{self, RunningApp};
use crate::stats::{self, DayTotal};
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

/// Start-of-local-day (unix seconds) for an arbitrary unix timestamp.
fn local_day_start(ts: i64) -> i64 {
    use chrono::{Datelike, Local, TimeZone, Timelike};
    match Local.timestamp_opt(ts, 0).single() {
        Some(dt) => Local
            .with_ymd_and_hms(dt.year(), dt.month(), dt.day(), 0, 0, 0)
            .earliest()
            .map(|m| m.timestamp())
            .unwrap_or_else(|| ts - dt.num_seconds_from_midnight() as i64),
        None => ts - ts.rem_euclid(86_400),
    }
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

#[tauri::command]
pub fn usage_range(state: State<AppState>, from: i64, to: i64) -> Result<Vec<UsageSlice>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::usage_between(&conn, from, to).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn daily_totals(state: State<AppState>, from: i64, to: i64) -> Result<Vec<DayTotal>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let rows = queries::sessions_between(&conn, from, to).map_err(|e| e.to_string())?;
    let pairs: Vec<(i64, i64)> = rows.iter().map(|r| (r.started_at, r.active_seconds)).collect();
    Ok(stats::daily_totals(&pairs, local_day_start))
}

#[tauri::command]
pub fn day_sessions(state: State<AppState>, from: i64, to: i64) -> Result<Vec<SessionRow>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::sessions_between(&conn, from, to).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_app_limit(state: State<AppState>, id: i64, daily_cap_seconds: i64, limit_action: String)
    -> Result<(), String> {
    let action = if limit_action == "close" { "close" } else { "warn" };
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::set_app_limit(&conn, id, daily_cap_seconds.max(0), action).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn snooze_limit(state: State<AppState>, exe: String, minutes: i64) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    let mut rt = state.limits.lock().map_err(|e| e.to_string())?;
    let s = rt.state_mut(&exe.to_lowercase());
    s.snoozed_until = now + minutes.max(1) * 60;
    s.warned = false;
    Ok(())
}

#[tauri::command]
pub fn ignore_limit(state: State<AppState>, exe: String) -> Result<(), String> {
    let mut rt = state.limits.lock().map_err(|e| e.to_string())?;
    rt.state_mut(&exe.to_lowercase()).ignored = true;
    Ok(())
}

#[tauri::command]
pub fn force_close(state: State<AppState>, exe: String) -> Result<(), String> {
    {
        let mut rt = state.limits.lock().map_err(|e| e.to_string())?;
        rt.state_mut(&exe.to_lowercase()).warned = true;
    }
    crate::proc::force_close(&exe).map_err(|e| e.to_string())
}
