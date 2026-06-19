use rusqlite::Connection;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct App {
    pub id: i64,
    pub display_name: String,
    pub exe_name: String,
    pub kind: String,
    pub color: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct UsageSlice {
    pub app_id: i64,
    pub display_name: String,
    pub color: String,
    pub kind: String,
    pub seconds: i64,
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
        "SELECT id, display_name, exe_name, kind, color FROM apps ORDER BY display_name COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], |r| Ok(App {
        id: r.get(0)?, display_name: r.get(1)?, exe_name: r.get(2)?,
        kind: r.get(3)?, color: r.get(4)?,
    }))?;
    rows.collect()
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
}
