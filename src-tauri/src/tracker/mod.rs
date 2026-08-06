pub mod session;
pub mod site;
#[cfg(windows)]
pub mod win;

use crate::autotrack::{self, AutoTracker};
use crate::db::queries;
use crate::limits::{self, Decision};
use crate::AppState;
use serde::Serialize;
use session::{Sample, SessionTracker};
use site::{SiteSample, SiteTracker};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const TICK_SECS: u64 = 1;
const IDLE_THRESHOLD_SECS: u64 = 300; // 5 minutes (configurable in a later part)
/// How recently the extension must have reported a domain for it to count.
/// The extension heartbeats every 30s, so 90s tolerates a couple of missed beats.
const SITE_STALE_SECS: i64 = 90;
/// Mirrors the frontend `colorFor` palette so auto-added apps look hand-picked.
const AUTO_PALETTE: &[&str] = &["#C2410C", "#7A6F5C", "#B8A98C", "#3A6EA5", "#2F6E4F", "#8A4FB3", "#B23A48"];

#[derive(Clone, Serialize)]
pub struct LiveStatus {
    pub exe: Option<String>,
    pub active_seconds: i64,
}

#[derive(Clone, Serialize)]
pub struct LimitEvent {
    pub kind: String, // "app" | "site"
    pub exe: String,
    pub display_name: String,
    pub cap_seconds: i64,
    pub today_seconds: i64,
    pub auto_close: bool,
}

fn local_today_start() -> i64 {
    use chrono::{Datelike, Local, TimeZone, Timelike};
    let now = Local::now();
    Local.with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .earliest().map(|d| d.timestamp())
        .unwrap_or_else(|| now.timestamp() - now.num_seconds_from_midnight() as i64)
}

fn fmt_hm(secs: i64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 { format!("{h}h {m}m") } else { format!("{m}m") }
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
        let mut watched: HashSet<String> = initial.into_iter().collect();
        let mut tracker = SessionTracker::new(watched.clone());
        let mut site_tracker = SiteTracker::new();
        let mut refresh_counter = 0u32;
        let mut limits_map: HashMap<String, (i64, String)> = HashMap::new();
        let mut site_limits_map: HashMap<String, (i64, String)> = HashMap::new();
        let mut site_day: i64 = 0;
        let mut auto = AutoTracker::new();
        let mut auto_enabled = false;
        let mut auto_threshold_secs: i64 = 600;
        let windows_dir = std::env::var("SystemRoot").unwrap_or_default();

        loop {
            std::thread::sleep(Duration::from_secs(TICK_SECS));

            // Refresh the watchlist every ~5s so newly-added apps start tracking.
            refresh_counter += 1;
            if refresh_counter >= 5 {
                refresh_counter = 0;
                if let Ok(conn) = state.db.lock() {
                    if let Ok(list) = queries::watched_exes(&conn) {
                        watched = list.into_iter().collect();
                        tracker.set_watched(watched.clone());
                    }
                    auto_enabled = matches!(
                        queries::get_setting(&conn, "autotrack_enabled").ok().flatten().as_deref(),
                        Some("true")
                    );
                    auto_threshold_secs = queries::get_setting(&conn, "autotrack_minutes")
                        .ok().flatten()
                        .and_then(|v| v.parse::<i64>().ok())
                        .filter(|m| *m > 0)
                        .unwrap_or(10) * 60;
                    if let Ok(list) = queries::app_limits(&conn) {
                        limits_map = list.into_iter().map(|(e, c, a)| (e, (c, a))).collect();
                    }
                    if let Ok(list) = queries::site_limits(&conn) {
                        site_limits_map = list.into_iter().map(|(d, c, a)| (d, (c, a))).collect();
                    }
                }

                let day_start = local_today_start();
                let total = state.db.lock().ok()
                    .and_then(|c| queries::total_seconds_between(&c, day_start, day_start + 86_400).ok())
                    .unwrap_or(0);
                if let Some(tray) = app.tray_by_id("main") {
                    let _ = tray.set_tooltip(Some(format!("Korio — Today: {}", fmt_hm(total))));
                }
            }

            let now = chrono::Utc::now().timestamp();
            let fg = win::foreground_app();
            let exe = fg.as_ref().map(|(e, _)| e.clone());
            let idle = win::idle_seconds() >= IDLE_THRESHOLD_SECS;

            // --- Auto-discovery: promote a long-focused, non-system app to the watchlist ---
            if auto_enabled && !idle {
                if let Some((fexe, fpath)) = fg.as_ref() {
                    if watched.contains(fexe) {
                        auto.forget(fexe);
                    } else if autotrack::is_auto_trackable(fexe, fpath, &windows_dir)
                        && auto.observe(local_today_start(), fexe, TICK_SECS as i64, auto_threshold_secs)
                    {
                        let name = autotrack::pretty_name(fexe);
                        let color = AUTO_PALETTE[watched.len() % AUTO_PALETTE.len()];
                        let added = state.db.lock().ok().and_then(|conn| {
                            queries::add_app(&conn, &name, fexe, "neutral", color, Some(fpath)).ok()
                        });
                        if added.is_some() {
                            // Start timing immediately instead of waiting for the next refresh.
                            watched.insert(fexe.clone());
                            tracker.set_watched(watched.clone());
                            let _ = app.emit("app-auto-added", &name);
                        }
                    }
                }
            }

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

            // --- Site (domain) tracking, parallel to app sessions ---
            let focused_domain = if !idle && exe.as_deref().map(crate::browser::is_browser_exe).unwrap_or(false) {
                crate::browser::current_fresh_domain(&app, SITE_STALE_SECS)
            } else {
                None
            };
            if let Some(finished) = site_tracker.observe(SiteSample { now, focused_domain }) {
                if let Ok(conn) = state.db.lock() {
                    let _ = queries::insert_site_session(
                        &conn, &finished.domain, finished.started_at, finished.ended_at, finished.seconds,
                    );
                }
            }

            // --- Per-site limit evaluation (parallel to app limits) ---
            // Clear the blocked set when the local day rolls over.
            let s_day = local_today_start();
            if s_day != site_day {
                site_day = s_day;
                if let Ok(mut b) = state.browser.lock() { b.blocked.clear(); }
            }
            if let Some((domain, sess_secs)) = site_tracker.current_domain().map(|(d, s)| (d.to_string(), s)) {
                if let Some((cap, action)) = site_limits_map.get(&domain).cloned() {
                    let today_db = state.db.lock().ok().and_then(|c|
                        queries::site_seconds_between(&c, &domain, s_day, s_day + 86_400).ok()
                    ).unwrap_or(0);
                    let today = today_db + sess_secs;
                    let now_unix = chrono::Utc::now().timestamp();
                    let decision = if let Ok(mut rt) = state.site_limits.lock() {
                        rt.reset_if_new_day(s_day);
                        let st = rt.state_mut(&domain);
                        let d = limits::decide(today, cap, &action, st, now_unix);
                        match d {
                            Decision::Warn => st.warned = true,
                            Decision::Close => st.close_requested = true,
                            Decision::None => {}
                        }
                        d
                    } else { Decision::None };
                    match decision {
                        Decision::Warn => {
                            crate::surface_main(&app);
                            let _ = app.emit("limit-reached", LimitEvent {
                                kind: "site".into(), exe: domain.clone(), display_name: domain.clone(),
                                cap_seconds: cap, today_seconds: today, auto_close: false,
                            });
                        }
                        Decision::Close => {
                            crate::surface_main(&app);
                            let _ = app.emit("limit-reached", LimitEvent {
                                kind: "site".into(), exe: domain.clone(), display_name: domain.clone(),
                                cap_seconds: cap, today_seconds: today, auto_close: true,
                            });
                        }
                        Decision::None => {}
                    }
                }
            }

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
                        match d {
                            Decision::Warn => st.warned = true,
                            Decision::Close => st.close_requested = true,
                            Decision::None => {}
                        }
                        d
                    } else { Decision::None };
                    match decision {
                        Decision::Warn => {
                            crate::surface_main(&app);
                            let name = display_name_for(&state, &exe).unwrap_or_else(|| exe.clone());
                            let _ = app.emit("limit-reached", LimitEvent {
                                kind: "app".into(), exe: exe.clone(), display_name: name,
                                cap_seconds: cap, today_seconds: today, auto_close: false,
                            });
                        }
                        Decision::Close => {
                            crate::surface_main(&app);
                            let name = display_name_for(&state, &exe).unwrap_or_else(|| exe.clone());
                            let _ = app.emit("limit-reached", LimitEvent {
                                kind: "app".into(), exe: exe.clone(), display_name: name,
                                cap_seconds: cap, today_seconds: today, auto_close: true,
                            });
                        }
                        Decision::None => {}
                    }
                }
            }
        }
    });
}
