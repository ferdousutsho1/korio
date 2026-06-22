pub mod queries;

use rusqlite::Connection;

/// Open a connection and ensure the schema exists.
pub fn open(path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    init_connection(&conn)?;
    migrate(&conn)?;
    Ok(conn)
}

/// Open an in-memory DB (used by tests).
pub fn open_in_memory() -> rusqlite::Result<Connection> {
    let conn = Connection::open_in_memory()?;
    init_connection(&conn)?;
    migrate(&conn)?;
    Ok(conn)
}

/// Connection-scoped pragmas. `foreign_keys` is OFF by default in SQLite and is
/// NOT persistent, so it must be enabled on every connection for the schema's
/// `ON DELETE CASCADE` to take effect.
fn init_connection(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")
}

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    // One-time reshape of the legacy unused stub `categories(id,name,kind)` to the new schema.
    // Only drops a categories table that lacks the new `nature` column; never touches an
    // already-migrated table (so user-created categories are preserved across boots).
    // - fresh DB      → no table → no drop, batch creates new schema
    // - legacy DB     → has id/name/kind but no `nature` → dropped, batch recreates new schema
    // - migrated DB   → has `nature` → not dropped, CREATE IF NOT EXISTS is a no-op
    {
        let mut stmt = conn.prepare("PRAGMA table_info(categories)")?;
        let cols: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(1))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let legacy = !cols.is_empty() && !cols.iter().any(|c| c == "nature");
        if legacy {
            conn.execute("DROP TABLE categories", [])?;
        }
    }

    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        CREATE TABLE IF NOT EXISTS categories (
            id         INTEGER PRIMARY KEY,
            name       TEXT NOT NULL UNIQUE,
            color      TEXT NOT NULL,
            nature     TEXT NOT NULL DEFAULT 'neutral'
                        CHECK (nature IN ('productive','neutral','distracting')),
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS apps (
            id                INTEGER PRIMARY KEY,
            display_name      TEXT NOT NULL,
            exe_name          TEXT NOT NULL UNIQUE,
            kind              TEXT NOT NULL DEFAULT 'neutral'
                               CHECK (kind IN ('productive','neutral','distracting')),
            color             TEXT NOT NULL DEFAULT '#C2410C',
            created_at        INTEGER NOT NULL,
            daily_cap_seconds INTEGER NOT NULL DEFAULT 0,
            limit_action      TEXT NOT NULL DEFAULT 'warn'
                               CHECK (limit_action IN ('warn','close'))
        );
        CREATE TABLE IF NOT EXISTS sessions (
            id             INTEGER PRIMARY KEY,
            app_id         INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
            started_at     INTEGER NOT NULL,
            ended_at       INTEGER NOT NULL,
            active_seconds INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at);
        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS tasks (
            id         INTEGER PRIMARY KEY,
            title      TEXT NOT NULL,
            done       INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS notes (
            id         INTEGER PRIMARY KEY,
            title      TEXT NOT NULL DEFAULT '',
            body       TEXT NOT NULL DEFAULT '',
            color      TEXT NOT NULL DEFAULT 'amber',
            due        TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS goals (
            id             INTEGER PRIMARY KEY,
            scope          TEXT NOT NULL CHECK (scope IN ('kind','app','total')),
            scope_ref      TEXT,
            comparator     TEXT NOT NULL CHECK (comparator IN ('gte','lte')),
            target_seconds INTEGER NOT NULL,
            created_at     INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS site_sessions (
            id         INTEGER PRIMARY KEY,
            domain     TEXT NOT NULL,
            started_at INTEGER NOT NULL,
            ended_at   INTEGER NOT NULL,
            seconds    INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_site_sessions_started ON site_sessions(started_at);
        CREATE INDEX IF NOT EXISTS idx_site_sessions_domain ON site_sessions(domain);
        CREATE TABLE IF NOT EXISTS sites (
            domain            TEXT PRIMARY KEY,
            daily_cap_seconds INTEGER NOT NULL DEFAULT 0,
            limit_action      TEXT NOT NULL DEFAULT 'warn'
                               CHECK (limit_action IN ('warn','close'))
        );
        "#,
    )?;
    add_column_if_missing(conn, "apps", "daily_cap_seconds", "INTEGER NOT NULL DEFAULT 0")?;
    add_column_if_missing(conn, "apps", "limit_action", "TEXT NOT NULL DEFAULT 'warn'")?;
    add_column_if_missing(conn, "apps", "exe_path", "TEXT")?;

    // One-time seed of default categories (gated so deleting them all doesn't resurrect them).
    if crate::db::queries::get_setting(conn, "categories_seeded")?.is_none() {
        let now = chrono::Utc::now().timestamp();
        for (name, color, nature) in [
            ("Productivity", "#2F6E4F", "productive"),
            ("Development", "#3A6EA5", "productive"),
            ("Communication", "#7A6F5C", "neutral"),
            ("Entertainment", "#B23A48", "distracting"),
        ] {
            conn.execute(
                "INSERT INTO categories (name, color, nature, created_at) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![name, color, nature, now],
            )?;
        }
        crate::db::queries::set_setting(conn, "categories_seeded", "true")?;
    }
    add_column_if_missing(conn, "apps", "category_id", "INTEGER")?;
    Ok(())
}

/// Add a column only if it isn't already present (SQLite ALTER lacks IF NOT EXISTS).
fn add_column_if_missing(conn: &Connection, table: &str, column: &str, decl: &str) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let existing: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if !existing.iter().any(|c| c == column) {
        conn.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {column} {decl};"))?;
    }
    Ok(())
}
