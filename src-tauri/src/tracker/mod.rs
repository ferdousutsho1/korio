pub mod session;
#[cfg(windows)]
pub mod win;

use crate::db::queries;
use crate::limits::{self, Decision};
use crate::AppState;
use serde::Serialize;
use session::{Sample, SessionTracker};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const TICK_SECS: u64 = 1;
const IDLE_THRESHOLD_SECS: u64 = 300; // 5 minutes (configurable in a later part)

#[derive(Clone, Serialize)]
pub struct LiveStatus {
    pub exe: Option<String>,
    pub active_seconds: i64,
}

#[derive(Clone, Serialize)]
pub struct LimitEvent {
    pub exe: String,
    pub display_name: String,
    pub cap_seconds: i64,
    pub today_seconds: i64,
}

fn local_today_start() -> i64 {
    use chrono::{Datelike, Local, TimeZone, Timelike};
    let now = Local::now();
    Local.with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .earliest().map(|d| d.timestamp())
        .unwrap_or_else(|| now.timestamp() - now.num_seconds_from_midnight() as i64)
}

fn display_name_for(state: &tauri::State<AppState>, exe: &str) -> Option<String> {
    let conn = state.db.lock().ok()?;
    conn.query_row("SELECT display_name FROM apps WHERE LOWER(exe_name) = LOWER(?1)", [exe], |r| r.get(0)).ok()
}

/// Spawn the background tracking loop. Runs for the life of the app.
pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        let state = app.state::<AppState>();
        let initial = {
            let conn = state.db.lock().unwrap();
            queries::watched_exes(&conn).unwrap_or_default()
        };
        let mut tracker = SessionTracker::new(initial.into_iter().collect::<HashSet<_>>());
        let mut refresh_counter = 0u32;
        let mut limits_map: HashMap<String, (i64, String)> = HashMap::new();

        loop {
            std::thread::sleep(Duration::from_secs(TICK_SECS));

            // Refresh the watchlist every ~5s so newly-added apps start tracking.
            refresh_counter += 1;
            if refresh_counter >= 5 {
                refresh_counter = 0;
                if let Ok(conn) = state.db.lock() {
                    if let Ok(list) = queries::watched_exes(&conn) {
                        tracker.set_watched(list.into_iter().collect());
                    }
                    if let Ok(list) = queries::app_limits(&conn) {
                        limits_map = list.into_iter().map(|(e, c, a)| (e, (c, a))).collect();
                    }
                }
            }

            let now = chrono::Utc::now().timestamp();
            let exe = win::foreground_exe();
            let idle = win::idle_seconds() >= IDLE_THRESHOLD_SECS;

            if let Some(finished) = tracker.observe(Sample { now, focused_exe: exe.clone(), idle }) {
                if let Ok(conn) = state.db.lock() {
                    let _ = queries::insert_session(
                        &conn, &finished.exe, finished.started_at,
                        finished.ended_at, finished.active_seconds,
                    );
                }
                let _ = app.emit("session-finished", &finished.exe);
            }

            let (live_exe, secs) = match tracker.current_exe() {
                Some((e, s)) => (Some(e.to_string()), s),
                None => (None, 0),
            };
            let _ = app.emit("live-status", LiveStatus { exe: live_exe, active_seconds: secs });

            if let Some((exe, sess_secs)) = tracker.current_exe().map(|(e, s)| (e.to_string(), s)) {
                if let Some((cap, action)) = limits_map.get(&exe).cloned() {
                    let day_start = local_today_start();
                    let today_db = state.db.lock().ok().and_then(|c|
                        queries::app_seconds_between(&c, &exe, day_start, day_start + 86_400).ok()
                    ).unwrap_or(0);
                    let today = today_db + sess_secs;
                    let now_unix = chrono::Utc::now().timestamp();
                    let decision = if let Ok(mut rt) = state.limits.lock() {
                        rt.reset_if_new_day(day_start);
                        let st = rt.state_mut(&exe);
                        let d = limits::decide(today, cap, &action, st, now_unix);
                        if d != Decision::None { st.warned = true; }
                        d
                    } else { Decision::None };
                    match decision {
                        Decision::Warn => {
                            let name = display_name_for(&state, &exe).unwrap_or_else(|| exe.clone());
                            let _ = app.emit("limit-reached", LimitEvent {
                                exe: exe.clone(), display_name: name, cap_seconds: cap, today_seconds: today,
                            });
                        }
                        Decision::Close => {
                            let _ = crate::proc::force_close(&exe);
                            let _ = app.emit("limit-closed", &exe);
                        }
                        Decision::None => {}
                    }
                }
            }
        }
    });
}
