use crate::utils::logger::{ask_input, info};
use anyhow::{Result, anyhow};
use chrono::Utc;
use colored::Colorize;
use rusqlite::{Connection, params};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub cache_dir: String,
    pub last_cleaned: String,
}

pub fn get_db_path() -> Result<PathBuf> {
    //in dev mode using the current dir
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
    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            name         TEXT NOT NULL,
            path         TEXT NOT NULL,
            cache_dir    TEXT NOT NULL,
            last_cleaned TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub fn add_project(name: &str, path: &str, cache_dir: &str) -> Result<i32> {
    let conn = connect()?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO projects (name, path, cache_dir, last_cleaned) VALUES (?1, ?2, ?3, ?4)",
        params![name, path, cache_dir, now],
    )?;
    Ok(conn.last_insert_rowid() as i32)
}

pub fn get_project_by_id(id: i32) -> Result<Option<Project>> {
    let conn = connect()?;
    let mut stmt =
        conn.prepare("SELECT id, name, path, cache_dir, last_cleaned FROM projects WHERE id = ?1")?;
    let project_iter = stmt.query_map(params![id], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            cache_dir: row.get(3)?,
            last_cleaned: row.get(4)?,
        })
    })?;

    for project in project_iter {
        return Ok(Some(project?));
    }
    Ok(None)
}

pub fn get_all_projects() -> Result<Vec<Project>> {
    let conn = connect()?;
    let mut stmt = conn.prepare("SELECT id, name, path, cache_dir, last_cleaned FROM projects")?;
    let project_iter = stmt.query_map([], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            cache_dir: row.get(3)?,
            last_cleaned: row.get(4)?,
        })
    })?;

    let mut projects = Vec::new();
    for project in project_iter {
        projects.push(project?);
    }
    Ok(projects)
}

pub fn update_last_cleaned(id: i32) -> Result<()> {
    let conn = connect()?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE projects SET last_cleaned = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    Ok(())
}

pub fn remove_project(id: i32) -> Result<()> {
    let conn = connect()?;
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn flush_db() -> Result<()> {
    let db_path = get_db_path()?;
    if db_path.exists() {
        fs::remove_file(&db_path)?;
        info("Database has been deleted!");
    } else {
        info("Database does not exists!");
    }
    Ok(())
}

//Helpers
pub fn project_exists(id: i32) -> Result<bool> {
    Ok(get_project_by_id(id)?.is_some())
}

//Tests

#[cfg(test)]
fn in_memory_conn() -> rusqlite::Result<rusqlite::Connection> {
    let conn = rusqlite::Connection::open_in_memory()?;
    conn.execute(
        "CREATE TABLE projects (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            name         TEXT NOT NULL,
            path         TEXT NOT NULL,
            cache_dir    TEXT NOT NULL,
            last_cleaned TEXT NOT NULL
        )",
        [],
    )?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_add_and_get_project() {
        let conn = in_memory_conn().unwrap();

        let name = "Test Project";
        let path = "/tmp/test";
        let cache_dir = "/tmp/test/cache";
        let last_cleaned = Utc::now().to_rfc3339();

        // Insert
        conn.execute(
            "INSERT INTO projects (name, path, cache_dir, last_cleaned) VALUES (?1, ?2, ?3, ?4)",
            params![name, path, cache_dir, last_cleaned],
        )
        .unwrap();

        // Get
        let mut stmt = conn
            .prepare("SELECT name, path, cache_dir, last_cleaned FROM projects")
            .unwrap();
        let project_iter = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: 0, // id not tested here
                    name: row.get(0)?,
                    path: row.get(1)?,
                    cache_dir: row.get(2)?,
                    last_cleaned: row.get(3)?,
                })
            })
            .unwrap();

        let projects: Vec<Project> = project_iter.map(|p| p.unwrap()).collect();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, name);
        assert_eq!(projects[0].path, path);
        assert_eq!(projects[0].cache_dir, cache_dir);
    }

    #[test]
    fn test_update_last_cleaned() {
        let conn = in_memory_conn().unwrap();

        conn.execute(
            "INSERT INTO projects (name, path, cache_dir, last_cleaned) VALUES (?1, ?2, ?3, ?4)",
            params!["Test", "/tmp", "/tmp/cache", "2025-01-01T00:00:00Z"],
        )
        .unwrap();

        let id: i32 = conn
            .query_row("SELECT id FROM projects", [], |row| row.get(0))
            .unwrap();

        // Update last_cleaned
        let new_time = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE projects SET last_cleaned = ?1 WHERE id = ?2",
            params![new_time, id],
        )
        .unwrap();

        let updated: String = conn
            .query_row(
                "SELECT last_cleaned FROM projects WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(updated, new_time);
    }

    #[test]
    fn test_remove_project() {
        let conn = in_memory_conn().unwrap();

        conn.execute(
            "INSERT INTO projects (name, path, cache_dir, last_cleaned) VALUES (?1, ?2, ?3, ?4)",
            params!["Test", "/tmp", "/tmp/cache", "2025-01-01T00:00:00Z"],
        )
        .unwrap();

        let id: i32 = conn
            .query_row("SELECT id FROM projects", [], |row| row.get(0))
            .unwrap();

        // Remove
        conn.execute("DELETE FROM projects WHERE id = ?1", [id])
            .unwrap();

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM projects", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
