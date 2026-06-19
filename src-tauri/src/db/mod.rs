pub mod queries;

use rusqlite::Connection;

/// Open a connection and ensure the schema exists.
pub fn open(path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    migrate(&conn)?;
    Ok(conn)
}

/// Open an in-memory DB (used by tests).
pub fn open_in_memory() -> rusqlite::Result<Connection> {
    let conn = Connection::open_in_memory()?;
    migrate(&conn)?;
    Ok(conn)
}

pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        CREATE TABLE IF NOT EXISTS categories (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL,
            kind  TEXT NOT NULL CHECK (kind IN ('productive','neutral','distracting'))
        );
        CREATE TABLE IF NOT EXISTS apps (
            id           INTEGER PRIMARY KEY,
            display_name TEXT NOT NULL,
            exe_name     TEXT NOT NULL UNIQUE,
            kind         TEXT NOT NULL DEFAULT 'neutral'
                          CHECK (kind IN ('productive','neutral','distracting')),
            color        TEXT NOT NULL DEFAULT '#C2410C',
            created_at   INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS sessions (
            id             INTEGER PRIMARY KEY,
            app_id         INTEGER NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
            started_at     INTEGER NOT NULL,
            ended_at       INTEGER NOT NULL,
            active_seconds INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_sessions_started ON sessions(started_at);
        "#,
    )
}
