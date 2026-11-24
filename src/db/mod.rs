pub mod queries;
pub mod schema;

use anyhow::Result;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

pub use queries::*;
pub use schema::*;

pub fn get_db_path() -> Result<PathBuf> {
    // Debug builds keep the DB in the working directory.
    if cfg!(debug_assertions) {
        return Ok(PathBuf::from("nspira.db"));
    }
    let dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nspira");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("nspira.db"))
}

/// Open the database with foreign keys enforced and the schema in place.
pub fn connect() -> Result<Connection> {
    let conn = Connection::open(get_db_path()?)?;
    prepare(&conn)?;
    Ok(conn)
}

pub(crate) fn prepare(conn: &Connection) -> Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(schema::SCHEMA)?;
    Ok(())
}

pub fn flush_db() -> Result<bool> {
    let db_path = get_db_path()?;
    if !db_path.exists() {
        return Ok(false);
    }
    fs::remove_file(&db_path)?;
    Ok(true)
}
