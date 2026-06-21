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
pub fn add_app(conn: &Connection, display_name: &str, exe_name: &str, kind: &str, color: &str, exe_path: Option<&str>)
    -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO apps (display_name, exe_name, kind, color, exe_path, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(exe_name) DO UPDATE SET
             display_name = excluded.display_name,
             exe_path = COALESCE(excluded.exe_path, apps.exe_path)",
        rusqlite::params![display_name, exe_name, kind, color, exe_path, now()],
    )?;
    conn.query_row(
        "SELECT id FROM apps WHERE exe_name = ?1",
        [exe_name],
        |r| r.get(0),
    )
}

/// (exe_name, exe_path) for launching, or None if the app id is unknown.
pub fn app_launch_info(conn: &Connection, id: i64) -> rusqlite::Result<Option<(String, Option<String>)>> {
    use rusqlite::OptionalExtension;
    conn.query_row(
        "SELECT exe_name, exe_path FROM apps WHERE id = ?1",
        [id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).optional()
}

pub fn set_app_path(conn: &Connection, id: i64, path: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE apps SET exe_path = ?2 WHERE id = ?1", rusqlite::params![id, path])?;
    Ok(())
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

#[derive(Debug, Serialize, Clone)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub done: bool,
    pub created_at: i64,
}

pub fn add_task(conn: &Connection, title: &str) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO tasks (title, done, created_at) VALUES (?1, 0, ?2)",
        rusqlite::params![title, now()],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Active tasks first (done last), then oldest-first within each group.
pub fn list_tasks(conn: &Connection) -> rusqlite::Result<Vec<Task>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, done, created_at FROM tasks ORDER BY done ASC, created_at ASC",
    )?;
    let rows = stmt.query_map([], |r| Ok(Task {
        id: r.get(0)?, title: r.get(1)?, done: r.get::<_, i64>(2)? != 0, created_at: r.get(3)?,
    }))?;
    rows.collect()
}

pub fn set_task_done(conn: &Connection, id: i64, done: bool) -> rusqlite::Result<()> {
    conn.execute("UPDATE tasks SET done = ?2 WHERE id = ?1", rusqlite::params![id, done as i64])?;
    Ok(())
}

pub fn update_task_title(conn: &Connection, id: i64, title: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE tasks SET title = ?2 WHERE id = ?1", rusqlite::params![id, title])?;
    Ok(())
}

pub fn delete_task(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM tasks WHERE id = ?1", [id])?;
    Ok(())
}

pub fn clear_done_tasks(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM tasks WHERE done = 1", [])?;
    Ok(())
}

fn now() -> i64 { chrono::Utc::now().timestamp() }

#[derive(Debug, Serialize, Clone)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub color: String,
    pub due: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Insert a blank note with a default color; returns its id.
pub fn add_note(conn: &Connection) -> rusqlite::Result<i64> {
    let t = now();
    conn.execute(
        "INSERT INTO notes (title, body, color, due, created_at, updated_at)
         VALUES ('', '', 'amber', NULL, ?1, ?1)",
        rusqlite::params![t],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Newest first (ties broken by id).
pub fn list_notes(conn: &Connection) -> rusqlite::Result<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, body, color, due, created_at, updated_at
         FROM notes ORDER BY created_at DESC, id DESC",
    )?;
    let rows = stmt.query_map([], |r| Ok(Note {
        id: r.get(0)?, title: r.get(1)?, body: r.get(2)?, color: r.get(3)?,
        due: r.get(4)?, created_at: r.get(5)?, updated_at: r.get(6)?,
    }))?;
    rows.collect()
}

pub fn update_note(
    conn: &Connection, id: i64, title: &str, body: &str, color: &str, due: Option<&str>,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE notes SET title = ?2, body = ?3, color = ?4, due = ?5, updated_at = ?6 WHERE id = ?1",
        rusqlite::params![id, title, body, color, due, now()],
    )?;
    Ok(())
}

pub fn delete_note(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
    Ok(())
}

#[derive(Debug, Serialize, Clone)]
pub struct Goal {
    pub id: i64,
    pub scope: String,            // 'kind' | 'app' | 'total'
    pub scope_ref: Option<String>,// kind name, or app_id as text, or NULL for total
    pub comparator: String,       // 'gte' | 'lte'
    pub target_seconds: i64,
    pub created_at: i64,
}

/// A session row carrying the fields goal-bucketing needs (incl. app_id, which SessionRow lacks).
pub struct GoalSession {
    pub started_at: i64,
    pub active_seconds: i64,
    pub kind: String,
    pub app_id: i64,
}

pub fn add_goal(conn: &Connection, scope: &str, scope_ref: Option<&str>, comparator: &str, target_seconds: i64)
    -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO goals (scope, scope_ref, comparator, target_seconds, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![scope, scope_ref, comparator, target_seconds, now()],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_goals(conn: &Connection) -> rusqlite::Result<Vec<Goal>> {
    let mut stmt = conn.prepare(
        "SELECT id, scope, scope_ref, comparator, target_seconds, created_at
         FROM goals ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([], |r| Ok(Goal {
        id: r.get(0)?, scope: r.get(1)?, scope_ref: r.get(2)?,
        comparator: r.get(3)?, target_seconds: r.get(4)?, created_at: r.get(5)?,
    }))?;
    rows.collect()
}

pub fn update_goal(conn: &Connection, id: i64, comparator: &str, target_seconds: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE goals SET comparator = ?2, target_seconds = ?3 WHERE id = ?1",
        rusqlite::params![id, comparator, target_seconds],
    )?;
    Ok(())
}

pub fn delete_goal(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM goals WHERE id = ?1", [id])?;
    Ok(())
}

/// Sessions in [from, to) with app kind + app_id, oldest first (for goal bucketing).
pub fn goal_sessions(conn: &Connection, from: i64, to: i64) -> rusqlite::Result<Vec<GoalSession>> {
    let mut stmt = conn.prepare(
        "SELECT s.started_at, s.active_seconds, a.kind, a.id
         FROM sessions s JOIN apps a ON a.id = s.app_id
         WHERE s.started_at >= ?1 AND s.started_at < ?2
         ORDER BY s.started_at ASC",
    )?;
    let rows = stmt.query_map([from, to], |r| Ok(GoalSession {
        started_at: r.get(0)?, active_seconds: r.get(1)?, kind: r.get(2)?, app_id: r.get(3)?,
    }))?;
    rows.collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn add_app_is_idempotent_on_exe() {
        let c = open_in_memory().unwrap();
        let id1 = add_app(&c, "Visual Studio Code", "code.exe", "productive", "#C2410C", None).unwrap();
        let id2 = add_app(&c, "VS Code", "code.exe", "productive", "#C2410C", None).unwrap();
        assert_eq!(id1, id2);
        assert_eq!(list_apps(&c).unwrap().len(), 1);
        assert_eq!(list_apps(&c).unwrap()[0].display_name, "VS Code");
    }

    #[test]
    fn usage_between_sums_active_seconds_in_window() {
        let c = open_in_memory().unwrap();
        add_app(&c, "VS Code", "code.exe", "productive", "#C2410C", None).unwrap();
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
        add_app(&c, "Chrome", "Chrome.EXE", "distracting", "#B8A98C", None).unwrap();
        assert_eq!(watched_exes(&c).unwrap(), vec!["chrome.exe".to_string()]);
    }

    #[test]
    fn removing_an_app_cascades_to_its_sessions() {
        let c = open_in_memory().unwrap();
        let id = add_app(&c, "VS Code", "code.exe", "productive", "#C2410C", None).unwrap();
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
        add_app(&c, "VS Code", "code.exe", "productive", "#C2410C", None).unwrap();
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
        let id = add_app(&c, "YouTube", "chrome.exe", "distracting", "#B23A48", None).unwrap();
        set_app_limit(&c, id, 1800, "close").unwrap();
        let limits = app_limits(&c).unwrap();
        assert_eq!(limits, vec![("chrome.exe".to_string(), 1800, "close".to_string())]);
        add_app(&c, "VS Code", "code.exe", "productive", "#C2410C", None).unwrap();
        assert_eq!(app_limits(&c).unwrap().len(), 1); // no-cap app excluded
    }

    #[test]
    fn app_seconds_between_sums_one_exe() {
        let c = open_in_memory().unwrap();
        add_app(&c, "VS Code", "code.exe", "productive", "#C2410C", None).unwrap();
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
        add_app(&c, "VS Code", "code.exe", "productive", "#C2410C", None).unwrap();
        add_app(&c, "Chrome", "chrome.exe", "distracting", "#B23A48", None).unwrap();
        insert_session(&c, "code.exe", 1000, 1100, 100).unwrap();
        insert_session(&c, "chrome.exe", 1200, 1300, 50).unwrap();
        insert_session(&c, "code.exe", 9000, 9100, 999).unwrap(); // outside
        assert_eq!(total_seconds_between(&c, 0, 2000).unwrap(), 150);
    }

    #[test]
    fn tasks_add_list_toggle_orders_active_first() {
        let c = open_in_memory().unwrap();
        let a = add_task(&c, "Write plan").unwrap();
        let _b = add_task(&c, "Review PR").unwrap();
        assert_eq!(list_tasks(&c).unwrap().len(), 2);
        set_task_done(&c, a, true).unwrap();
        let list = list_tasks(&c).unwrap();
        assert_eq!(list[0].title, "Review PR"); // active first
        assert_eq!(list[1].title, "Write plan");
        assert!(list[1].done);
    }

    #[test]
    fn tasks_update_delete_and_clear_done() {
        let c = open_in_memory().unwrap();
        let a = add_task(&c, "typo ttile").unwrap();
        let b = add_task(&c, "done me").unwrap();
        update_task_title(&c, a, "fixed title").unwrap();
        assert_eq!(list_tasks(&c).unwrap().iter().find(|t| t.id == a).unwrap().title, "fixed title");
        set_task_done(&c, b, true).unwrap();
        clear_done_tasks(&c).unwrap();
        let list = list_tasks(&c).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, a);
        delete_task(&c, a).unwrap();
        assert!(list_tasks(&c).unwrap().is_empty());
    }

    #[test]
    fn notes_add_defaults_and_lists_newest_first() {
        let c = open_in_memory().unwrap();
        let a = add_note(&c).unwrap();
        let b = add_note(&c).unwrap();
        let list = list_notes(&c).unwrap();
        assert_eq!(list.len(), 2);
        // newest (higher id) first when created_at ties
        assert_eq!(list[0].id, b);
        assert_eq!(list[1].id, a);
        // defaults
        assert_eq!(list[0].title, "");
        assert_eq!(list[0].body, "");
        assert_eq!(list[0].color, "amber");
        assert!(list[0].due.is_none());
    }

    #[test]
    fn notes_update_and_delete() {
        let c = open_in_memory().unwrap();
        let a = add_note(&c).unwrap();
        update_note(&c, a, "Groceries", "milk\neggs", "sage", Some("2026-07-01")).unwrap();
        let n = list_notes(&c).unwrap().into_iter().find(|n| n.id == a).unwrap();
        assert_eq!(n.title, "Groceries");
        assert_eq!(n.body, "milk\neggs");
        assert_eq!(n.color, "sage");
        assert_eq!(n.due.as_deref(), Some("2026-07-01"));
        // clearing the due date
        update_note(&c, a, "Groceries", "milk\neggs", "sage", None).unwrap();
        assert!(list_notes(&c).unwrap()[0].due.is_none());
        delete_note(&c, a).unwrap();
        assert!(list_notes(&c).unwrap().is_empty());
    }

    #[test]
    fn goals_crud_roundtrip() {
        let c = open_in_memory().unwrap();
        let id = add_goal(&c, "kind", Some("productive"), "gte", 7200).unwrap();
        add_goal(&c, "total", None, "gte", 14400).unwrap();
        let all = list_goals(&c).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].scope, "kind");
        assert_eq!(all[0].scope_ref.as_deref(), Some("productive"));
        assert_eq!(all[0].target_seconds, 7200);
        update_goal(&c, id, "lte", 1800).unwrap();
        let g = list_goals(&c).unwrap().into_iter().find(|g| g.id == id).unwrap();
        assert_eq!(g.comparator, "lte");
        assert_eq!(g.target_seconds, 1800);
        delete_goal(&c, id).unwrap();
        assert_eq!(list_goals(&c).unwrap().len(), 1);
    }
}
