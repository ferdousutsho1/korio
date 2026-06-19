pub mod session;
#[cfg(windows)]
pub mod win;

use crate::db::queries;
use crate::AppState;
use serde::Serialize;
use session::{Sample, SessionTracker};
use std::collections::HashSet;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const TICK_SECS: u64 = 1;
const IDLE_THRESHOLD_SECS: u64 = 300; // 5 minutes (configurable in a later part)

#[derive(Clone, Serialize)]
pub struct LiveStatus {
    pub exe: Option<String>,
    pub active_seconds: i64,
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
        }
    });
}
