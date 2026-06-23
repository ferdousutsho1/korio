use crate::db::queries::{self, App, Category, CategoryUsage, Goal, GoalSession, Note, SessionRow, Task, UsageSlice};
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
    exe_path: Option<String>,
) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::add_app(&conn, &display_name, &exe_name, &kind, &color, exe_path.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn launch_app(state: State<AppState>, id: i64) -> Result<(), String> {
    let (exe_name, exe_path) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        queries::app_launch_info(&conn, id).map_err(|e| e.to_string())?
            .ok_or_else(|| "app not found".to_string())?
    };
    // 1) stored path, if it still exists
    if let Some(p) = exe_path.as_deref() {
        if std::path::Path::new(p).exists() {
            return crate::proc::launch_path(p).map_err(|e| e.to_string());
        }
    }
    // 2) resolve from a running process, and persist what we found
    if let Some(p) = crate::discovery::path_for_exe(&exe_name) {
        if let Ok(conn) = state.db.lock() {
            let _ = queries::set_app_path(&conn, id, &p);
        }
        return crate::proc::launch_path(&p).map_err(|e| e.to_string());
    }
    // 3) shell fallback by name
    crate::proc::launch_by_name(&exe_name).map_err(|e| e.to_string())
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
pub fn site_usage_today(state: State<AppState>) -> Result<Vec<queries::SiteUsage>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (from, to) = day_bounds_local();
    queries::site_usage_between(&conn, from, to).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn site_usage_range(state: State<AppState>, from: i64, to: i64) -> Result<Vec<queries::SiteUsage>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::site_usage_between(&conn, from, to).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_site(state: State<AppState>, domain: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::clear_site(&conn, &domain).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_site_caps(state: State<AppState>) -> Result<Vec<queries::SiteCap>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_site_caps(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_site_limit(state: State<AppState>, domain: String, daily_cap_seconds: i64, limit_action: String)
    -> Result<(), String> {
    {
        let conn = state.db.lock().map_err(|e| e.to_string())?;
        queries::set_site_limit(&conn, &domain, daily_cap_seconds, &limit_action).map_err(|e| e.to_string())?;
    }
    if daily_cap_seconds == 0 {
        if let Ok(mut b) = state.browser.lock() { b.blocked.remove(&domain); }
        // Also drop any runtime limit state (warned/snoozed/ignored) so re-adding a
        // cap for this domain later today warns fresh instead of staying suppressed.
        if let Ok(mut rt) = state.site_limits.lock() { rt.states.remove(&domain); }
    }
    Ok(())
}

#[tauri::command]
pub fn snooze_site_limit(state: State<AppState>, domain: String, minutes: i64) -> Result<(), String> {
    let mut rt = state.site_limits.lock().map_err(|e| e.to_string())?;
    rt.state_mut(&domain).snoozed_until = chrono::Utc::now().timestamp() + minutes * 60;
    Ok(())
}

#[tauri::command]
pub fn ignore_site_limit(state: State<AppState>, domain: String) -> Result<(), String> {
    {
        let mut rt = state.site_limits.lock().map_err(|e| e.to_string())?;
        rt.state_mut(&domain).ignored = true;
    }
    if let Ok(mut b) = state.browser.lock() { b.blocked.remove(&domain); }
    Ok(())
}

#[tauri::command]
pub fn block_site(state: State<AppState>, domain: String) -> Result<(), String> {
    if let Ok(mut b) = state.browser.lock() { b.blocked.insert(domain); }
    Ok(())
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

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<std::collections::HashMap<String, String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    Ok(queries::all_settings(&conn).map_err(|e| e.to_string())?.into_iter().collect())
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::set_setting(&conn, &key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_data(state: State<AppState>, path: String, format: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let rows = queries::export_rows(&conn).map_err(|e| e.to_string())?;
    let contents = if format == "json" { crate::export::to_json(&rows) } else { crate::export::to_csv(&rows) };
    std::fs::write(&path, contents).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn backup_db(state: State<AppState>, path: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::backup::backup_to(&conn, &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn restore_db(state: State<AppState>, path: String) -> Result<(), String> {
    let mut conn = state.db.lock().map_err(|e| e.to_string())?;
    crate::backup::restore_from(&mut conn, &path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn has_pin(state: State<AppState>) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let salt = queries::get_setting(&conn, "pin_salt").map_err(|e| e.to_string())?;
    let hash = queries::get_setting(&conn, "pin_hash").map_err(|e| e.to_string())?;
    Ok(salt.is_some() && hash.is_some())
}

#[tauri::command]
pub fn set_pin(state: State<AppState>, pin: String) -> Result<(), String> {
    let salt = crate::lock::new_salt();
    let hash = crate::lock::hash_pin(&pin, &salt);
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::set_setting(&conn, "pin_salt", &salt).map_err(|e| e.to_string())?;
    queries::set_setting(&conn, "pin_hash", &hash).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn verify_pin(state: State<AppState>, pin: String) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let salt = queries::get_setting(&conn, "pin_salt").map_err(|e| e.to_string())?;
    let hash = queries::get_setting(&conn, "pin_hash").map_err(|e| e.to_string())?;
    match (salt, hash) {
        (Some(s), Some(h)) => Ok(crate::lock::verify_pin(&pin, &s, &h)),
        _ => Ok(true),
    }
}

#[tauri::command]
pub fn list_tasks(state: State<AppState>, from: Option<i64>, to: Option<i64>) -> Result<Vec<Task>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    match (from, to) {
        (Some(f), Some(t)) => queries::list_tasks_between(&conn, f, t).map_err(|e| e.to_string()),
        _ => queries::list_tasks(&conn).map_err(|e| e.to_string()),
    }
}

#[tauri::command]
pub fn add_task(state: State<AppState>, title: String, created_at: Option<i64>) -> Result<i64, String> {
    let t = title.trim();
    if t.is_empty() { return Err("empty title".into()); }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    match created_at {
        Some(ts) => queries::add_task_at(&conn, t, ts).map_err(|e| e.to_string()),
        None => queries::add_task(&conn, t).map_err(|e| e.to_string()),
    }
}

#[tauri::command]
pub fn set_task_done(state: State<AppState>, id: i64, done: bool) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::set_task_done(&conn, id, done).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_task_title(state: State<AppState>, id: i64, title: String) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::update_task_title(&conn, id, title.trim()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_task(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::delete_task(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_done_tasks(state: State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::clear_done_tasks(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_pin(state: State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM settings WHERE key IN ('pin_salt','pin_hash')", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_notes(state: State<AppState>) -> Result<Vec<Note>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_notes(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_note(state: State<AppState>) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::add_note(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_note(
    state: State<AppState>, id: i64, title: String, body: String, color: String, due: Option<String>,
) -> Result<(), String> {
    let due = due.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::update_note(&conn, id, title.trim(), &body, color.trim(), due).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_note(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::delete_note(&conn, id).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct GoalProgress {
    pub goal: Goal,
    pub today_seconds: i64,
    pub met_today: bool,
    pub current_streak: i64,
    pub best_streak: i64,
}

const GOAL_LOOKBACK_DAYS: i64 = 365;

/// Per-local-day seconds for a goal, oldest→newest with the last element = today.
/// Days run from the goal's creation day (clamped to the lookback window) through today,
/// zero-filled for inactive days so streaks see gaps.
fn build_daily(sessions: &[GoalSession], g: &Goal, today_start: i64) -> Vec<i64> {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<i64, i64> = BTreeMap::new();
    for s in sessions {
        let matches = match g.scope.as_str() {
            "app" => g.scope_ref.as_deref() == Some(s.app_id.to_string().as_str()),
            "kind" => g.scope_ref.as_deref() == Some(s.kind.as_str()),
            _ => true, // total
        };
        if matches {
            *map.entry(local_day_start(s.started_at)).or_insert(0) += s.active_seconds;
        }
    }
    let earliest = local_day_start(today_start - GOAL_LOOKBACK_DAYS * 86_400);
    let start = local_day_start(g.created_at).max(earliest);
    let mut days: Vec<i64> = Vec::new();
    let mut cur = start;
    while cur <= today_start {
        days.push(cur);
        // +25h then snap to local midnight → robustly advances exactly one calendar day across DST
        cur = local_day_start(cur + 86_400 + 3_600);
    }
    days.iter().map(|d| *map.get(d).unwrap_or(&0)).collect()
}

#[tauri::command]
pub fn list_goals(state: State<AppState>) -> Result<Vec<Goal>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_goals(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_goal(
    state: State<AppState>, scope: String, scope_ref: Option<String>, comparator: String, target_seconds: i64,
) -> Result<i64, String> {
    if !matches!(scope.as_str(), "kind" | "app" | "total") { return Err("bad scope".into()); }
    if !matches!(comparator.as_str(), "gte" | "lte") { return Err("bad comparator".into()); }
    if target_seconds <= 0 { return Err("target must be positive".into()); }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::add_goal(&conn, &scope, scope_ref.as_deref(), &comparator, target_seconds).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_goal(
    state: State<AppState>, id: i64, comparator: String, target_seconds: i64,
) -> Result<(), String> {
    if !matches!(comparator.as_str(), "gte" | "lte") { return Err("bad comparator".into()); }
    if target_seconds <= 0 { return Err("target must be positive".into()); }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::update_goal(&conn, id, &comparator, target_seconds).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_goal(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::delete_goal(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn goals_progress(state: State<AppState>) -> Result<Vec<GoalProgress>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let goals = queries::list_goals(&conn).map_err(|e| e.to_string())?;
    let (today_start, today_end) = day_bounds_local();
    let from = today_start - GOAL_LOOKBACK_DAYS * 86_400;
    let sessions = queries::goal_sessions(&conn, from, today_end).map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(goals.len());
    for g in goals {
        let daily = build_daily(&sessions, &g, today_start);
        let gte = g.comparator == "gte";
        let s = crate::goals::streaks(&daily, g.target_seconds, gte);
        let today_seconds = *daily.last().unwrap_or(&0);
        out.push(GoalProgress {
            today_seconds,
            met_today: crate::goals::met(today_seconds, g.target_seconds, gte),
            current_streak: s.current,
            best_streak: s.best,
            goal: g,
        });
    }
    Ok(out)
}

#[tauri::command]
pub fn list_categories(state: State<AppState>) -> Result<Vec<Category>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::list_categories(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_category(state: State<AppState>, name: String, color: String, nature: String) -> Result<i64, String> {
    let n = name.trim();
    if n.is_empty() { return Err("empty name".into()); }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::add_category(&conn, n, &color, &nature).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_category(state: State<AppState>, id: i64, name: String, color: String, nature: String) -> Result<(), String> {
    let n = name.trim();
    if n.is_empty() { return Err("empty name".into()); }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::update_category(&conn, id, n, &color, &nature).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_category(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::delete_category(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_app_category(state: State<AppState>, id: i64, category_id: Option<i64>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::set_app_category(&conn, id, category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn usage_by_category(state: State<AppState>, from: i64, to: i64) -> Result<Vec<CategoryUsage>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::usage_by_category(&conn, from, to).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_note_size(state: State<AppState>, id: i64, width: i64, height: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    queries::set_note_size(&conn, id, width.max(0), height.max(0)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_alert_topmost(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    if let Some(w) = app.get_webview_window("main") {
        w.set_always_on_top(false).map_err(|e| e.to_string())?;
    }
    Ok(())
}
