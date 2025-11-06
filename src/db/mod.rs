pub mod queries;
pub mod schema;

use anyhow::{Result, anyhow};
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;

pub use queries::*;
pub use schema::*;

pub fn get_db_path() -> Result<PathBuf> {
    // In dev mode using the current dir
    if cfg!(debug_assertions) {
        Ok(PathBuf::from("nspira.db"))
    } else {
        let dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        let nspira_dir = dir.join("nspira");
        if !nspira_dir.exists() {
            fs::create_dir_all(&nspira_dir)?;
        }
        Ok(nspira_dir.join("nspira.db"))
    }
}

pub fn connect() -> Result<Connection> {
    let path = get_db_path()?;
    Connection::open(path).map_err(|e| anyhow!(e))
}

pub fn init_db() -> Result<()> {
    let conn = connect()?;

    // Create tables
    conn.execute(schema::CREATE_PROJECTS_TABLE, [])?;
    conn.execute(schema::CREATE_CACHE_DIRECTORIES_TABLE, [])?;
    conn.execute(schema::CREATE_CLEANING_HISTORY_TABLE, [])?;

    // Create indices
    conn.execute(schema::CREATE_PROJECTS_INDEX, [])?;
    conn.execute(schema::CREATE_CACHE_DIRS_INDEX, [])?;
    conn.execute(schema::CREATE_HISTORY_INDEX, [])?;
    conn.execute(schema::CREATE_HISTORY_DATE_INDEX, [])?;

    Ok(())
}

pub fn flush_db() -> Result<()> {
    let db_path = get_db_path()?;
    if db_path.exists() {
        fs::remove_file(&db_path)?;
        crate::utils::logger::info("Database has been deleted!");
    } else {
        crate::utils::logger::info("Database does not exist!");
    }
    Ok(())
}
