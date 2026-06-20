use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct App {
    pub id: i64,
    pub display_name: String,
    pub exe_name: String,
    pub kind: String,
    pub color: String,
    pub daily_cap_seconds: i64,
    pub limit_action: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct UsageSlice {
    pub app_id: i64,
    pub display_name: String,
    pub color: String,
    pub kind: String,
    pub seconds: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct SessionRow {
    pub display_name: String,
    pub color: String,
    pub kind: String,
    pub started_at: i64,
    pub ended_at: i64,
    pub active_seconds: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct ExportRow {
    pub app: String,
    pub exe: String,
    pub kind: String,
    pub started_at: i64,
    pub ended_at: i64,
    pub active_seconds: i64,
}

/// All sessions (all time), newest first, with their app info — for export.
pub fn export_rows(conn: &Connection) -> rusqlite::Result<Vec<ExportRow>> {
    let mut stmt = conn.prepare(
        "SELECT a.display_name, a.exe_name, a.kind, s.started_at, s.ended_at, s.active_seconds
         FROM sessions s JOIN apps a ON a.id = s.app_id
         ORDER BY s.started_at DESC",
    )?;
    let rows = stmt.query_map([], |r| Ok(ExportRow {
        app: r.get(0)?, exe: r.get(1)?, kind: r.get(2)?,
        started_at: r.get(3)?, ended_at: r.get(4)?, active_seconds: r.get(5)?,
    }))?;
    rows.collect()
}

/// Insert a watchlist app (idempotent on exe_name). Returns the row id.
pub fn add_app(conn: &Connection, display_name: &str, exe_name: &str, kind: &str, color: &str)
    -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO apps (display_name, exe_name, kind, color, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(exe_name) DO UPDATE SET display_name = excluded.display_name",
        rusqlite::params![display_name, exe_name, kind, color, now()],
    )?;
    conn.query_row(
        "SELECT id FROM apps WHERE exe_name = ?1",
        [exe_name],
        |r| r.get(0),
    )
}

pub fn remove_app(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM apps WHERE id = ?1", [id])?;
    Ok(())
}

pub fn list_apps(conn: &Connection) -> rusqlite::Result<Vec<App>> {
    let mut stmt = conn.prepare(
        "SELECT id, display_name, exe_name, kind, color, daily_cap_seconds, limit_action
         FROM apps ORDER BY display_name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], |r| Ok(App {
        id: r.get(0)?, display_name: r.get(1)?, exe_name: r.get(2)?,
        kind: r.get(3)?, color: r.get(4)?,
        daily_cap_seconds: r.get(5)?, limit_action: r.get(6)?,
    }))?;
    rows.collect()
}

/// Set the daily cap (seconds; 0 = no limit) and action ('warn'|'close') for an app.
pub fn set_app_limit(conn: &Connection, id: i64, daily_cap_seconds: i64, limit_action: &str)
    -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE apps SET daily_cap_seconds = ?2, limit_action = ?3 WHERE id = ?1",
        rusqlite::params![id, daily_cap_seconds, limit_action],
    )?;
    Ok(())
}

/// (lowercased exe, cap_seconds, action) for every app that has a cap (> 0).
pub fn app_limits(conn: &Connection) -> rusqlite::Result<Vec<(String, i64, String)>> {
    let mut stmt = conn.prepare(
        "SELECT LOWER(exe_name), daily_cap_seconds, limit_action FROM apps WHERE daily_cap_seconds > 0",
    )?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
    rows.collect()
}

/// Total active seconds for one exe within [from, to).
pub fn app_seconds_between(conn: &Connection, exe: &str, from: i64, to: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(SUM(s.active_seconds), 0)
         FROM sessions s JOIN apps a ON a.id = s.app_id
         WHERE LOWER(a.exe_name) = LOWER(?1) AND s.started_at >= ?2 AND s.started_at < ?3",
        rusqlite::params![exe, from, to],
        |r| r.get(0),
    )
}

/// All watched exe names, lower-cased, for the tracker's "is this watched?" check.
pub fn watched_exes(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT LOWER(exe_name) FROM apps")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    rows.collect()
}

pub fn insert_session(conn: &Connection, exe_name: &str, started_at: i64, ended_at: i64, active_seconds: i64)
    -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO sessions (app_id, started_at, ended_at, active_seconds)
         SELECT id, ?2, ?3, ?4 FROM apps WHERE LOWER(exe_name) = LOWER(?1)",
        rusqlite::params![exe_name, started_at, ended_at, active_seconds],
    )?;
    Ok(())
}

/// Aggregate active seconds per app within [from, to) (unix seconds).
pub fn usage_between(conn: &Connection, from: i64, to: i64) -> rusqlite::Result<Vec<UsageSlice>> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.display_name, a.color, a.kind, COALESCE(SUM(s.active_seconds),0) AS secs
         FROM apps a
         LEFT JOIN sessions s ON s.app_id = a.id AND s.started_at >= ?1 AND s.started_at < ?2
         GROUP BY a.id
         HAVING secs > 0
         ORDER BY secs DESC",
    )?;
    let rows = stmt.query_map([from, to], |r| Ok(UsageSlice {
        app_id: r.get(0)?, display_name: r.get(1)?, color: r.get(2)?,
        kind: r.get(3)?, seconds: r.get(4)?,
    }))?;
    rows.collect()
}

/// Raw sessions whose start falls within [from, to), joined with their app, oldest first.
pub fn sessions_between(conn: &Connection, from: i64, to: i64) -> rusqlite::Result<Vec<SessionRow>> {
    let mut stmt = conn.prepare(
        "SELECT a.display_name, a.color, a.kind, s.started_at, s.ended_at, s.active_seconds
         FROM sessions s JOIN apps a ON a.id = s.app_id
         WHERE s.started_at >= ?1 AND s.started_at < ?2
         ORDER BY s.started_at ASC",
    )?;
    let rows = stmt.query_map([from, to], |r| Ok(SessionRow {
        display_name: r.get(0)?, color: r.get(1)?, kind: r.get(2)?,
        started_at: r.get(3)?, ended_at: r.get(4)?, active_seconds: r.get(5)?,
    }))?;
    rows.collect()
}

pub fn get_setting(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| r.get(0))
        .map(Some)
        .or_else(|e| if e == rusqlite::Error::QueryReturnedNoRows { Ok(None) } else { Err(e) })
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

pub fn all_settings(conn: &Connection) -> rusqlite::Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

/// Total active seconds across ALL apps within [from, to).
pub fn total_seconds_between(conn: &Connection, from: i64, to: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(SUM(active_seconds), 0) FROM sessions WHERE started_at >= ?1 AND started_at < ?2",
        rusqlite::params![from, to],
        |r| r.get(0),
    )
}

fn now() -> i64 { chrono::Utc::now().timestamp() }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn add_app_is_idempotent_on_exe() {
        let c = open_in_memory().unwrap();
        let id1 = add_app(&c, "Visual Studio Code", "code.exe", "productive", "#C2410C").unwrap();
        let id2 = add_app(&c, "VS Code", "code.exe", "productive", "#C2410C").unwrap();
        assert_eq!(id1, id2);
        assert_eq!(list_apps(&c).unwrap().len(), 1);
        assert_eq!(list_apps(&c).unwrap()[0].display_name, "VS Code");
    }

    #[test]
    fn usage_between_sums_active_seconds_in_window() {
        let c = open_in_memory().unwrap();
        add_app(&c, "VS Code", "code.exe", "productive", "#C2410C").unwrap();
        insert_session(&c, "code.exe", 1000, 1100, 90).unwrap();
        insert_session(&c, "code.exe", 1200, 1300, 60).unwrap();
        insert_session(&c, "code.exe", 5000, 5100, 999).unwrap(); // outside window
        let usage = usage_between(&c, 0, 2000).unwrap();
        assert_eq!(usage.len(), 1);
        assert_eq!(usage[0].seconds, 150);
    }

    #[test]
    fn watched_exes_are_lowercased() {
        let c = open_in_memory().unwrap();
        add_app(&c, "Chrome", "Chrome.EXE", "distracting", "#B8A98C").unwrap();
        assert_eq!(watched_exes(&c).unwrap(), vec!["chrome.exe".to_string()]);
    }

    #[test]
    fn removing_an_app_cascades_to_its_sessions() {
        let c = open_in_memory().unwrap();
        let id = add_app(&c, "VS Code", "code.exe", "productive", "#C2410C").unwrap();
        insert_session(&c, "code.exe", 1000, 1100, 90).unwrap();
        remove_app(&c, id).unwrap();
        // With PRAGMA foreign_keys = ON, the ON DELETE CASCADE must remove sessions too.
        let orphans: i64 = c
            .query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(orphans, 0, "sessions should cascade-delete with their app");
    }

    #[test]
    fn sessions_between_returns_rows_in_window_oldest_first() {
        let c = open_in_memory().unwrap();
        add_app(&c, "VS Code", "code.exe", "productive", "#C2410C").unwrap();
        insert_session(&c, "code.exe", 1200, 1260, 60).unwrap();
        insert_session(&c, "code.exe", 1000, 1100, 90).unwrap();
        insert_session(&c, "code.exe", 5000, 5100, 10).unwrap(); // outside
        let rows = sessions_between(&c, 0, 2000).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].started_at, 1000); // oldest first
        assert_eq!(rows[1].started_at, 1200);
        assert_eq!(rows[0].display_name, "VS Code");
    }

    #[test]
    fn set_app_limit_and_app_limits_roundtrip() {
        let c = open_in_memory().unwrap();
        let id = add_app(&c, "YouTube", "chrome.exe", "distracting", "#B23A48").unwrap();
        set_app_limit(&c, id, 1800, "close").unwrap();
        let limits = app_limits(&c).unwrap();
        assert_eq!(limits, vec![("chrome.exe".to_string(), 1800, "close".to_string())]);
        add_app(&c, "VS Code", "code.exe", "productive", "#C2410C").unwrap();
        assert_eq!(app_limits(&c).unwrap().len(), 1); // no-cap app excluded
    }

    #[test]
    fn app_seconds_between_sums_one_exe() {
        let c = open_in_memory().unwrap();
        add_app(&c, "VS Code", "code.exe", "productive", "#C2410C").unwrap();
        insert_session(&c, "code.exe", 1000, 1100, 100).unwrap();
        insert_session(&c, "code.exe", 1200, 1300, 50).unwrap();
        assert_eq!(app_seconds_between(&c, "CODE.EXE", 0, 2000).unwrap(), 150);
        assert_eq!(app_seconds_between(&c, "code.exe", 0, 1150).unwrap(), 100);
    }

    #[test]
    fn settings_get_set_roundtrip_and_default_none() {
        let c = open_in_memory().unwrap();
        assert_eq!(get_setting(&c, "autostart").unwrap(), None);
        set_setting(&c, "autostart", "true").unwrap();
        assert_eq!(get_setting(&c, "autostart").unwrap(), Some("true".to_string()));
        set_setting(&c, "autostart", "false").unwrap();
        assert_eq!(get_setting(&c, "autostart").unwrap(), Some("false".to_string()));
    }

    #[test]
    fn total_seconds_between_sums_all_apps() {
        let c = open_in_memory().unwrap();
        add_app(&c, "VS Code", "code.exe", "productive", "#C2410C").unwrap();
        add_app(&c, "Chrome", "chrome.exe", "distracting", "#B23A48").unwrap();
        insert_session(&c, "code.exe", 1000, 1100, 100).unwrap();
        insert_session(&c, "chrome.exe", 1200, 1300, 50).unwrap();
        insert_session(&c, "code.exe", 9000, 9100, 999).unwrap(); // outside
        assert_eq!(total_seconds_between(&c, 0, 2000).unwrap(), 150);
    }
}
