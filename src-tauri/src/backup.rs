use rusqlite::{backup::Backup, Connection};
use std::time::Duration;

pub fn backup_to(conn: &Connection, dest: &str) -> rusqlite::Result<()> {
    let mut dst = Connection::open(dest)?;
    let bk = Backup::new(conn, &mut dst)?;
    bk.run_to_completion(100, Duration::from_millis(50), None)?;
    Ok(())
}

pub fn restore_from(conn: &mut Connection, src: &str) -> rusqlite::Result<()> {
    let src_conn = Connection::open(src)?;
    let bk = Backup::new(&src_conn, conn)?;
    bk.run_to_completion(100, Duration::from_millis(50), None)?;
    Ok(())
}
