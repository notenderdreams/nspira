use anyhow::Result;
use chrono::Utc;
use rusqlite::{Connection, params};

use super::schema::*;

// Project queries
pub fn add_project(conn: &Connection, name: &str, path: &str) -> Result<i32> {
    conn.execute(
        "INSERT INTO projects (name, path) VALUES (?1, ?2)",
        params![name, path],
    )?;
    Ok(conn.last_insert_rowid() as i32)
}

pub fn get_project_by_id(conn: &Connection, id: i32) -> Result<Option<Project>> {
    let mut stmt =
        conn.prepare("SELECT id, name, path, last_cleaned FROM projects WHERE id = ?1")?;
    let mut project_iter = stmt.query_map(params![id], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            last_cleaned: row
                .get::<_, Option<String>>(3)?
                .unwrap_or_else(|| "Never".to_string()),
            cache_dirs: Vec::new(),
        })
    })?;

    if let Some(project) = project_iter.next() {
        let mut project = project?;
        // attach cache_dirs
        let cache_dirs = get_cache_directories_for_project(conn, project.id)?;
        project.cache_dirs = cache_dirs.iter().map(|cd| cd.path.clone()).collect();
        return Ok(Some(project));
    }
    Ok(None)
}

pub fn get_project_by_path(conn: &Connection, path: &str) -> Result<Option<Project>> {
    let mut stmt =
        conn.prepare("SELECT id, name, path, last_cleaned FROM projects WHERE path = ?1")?;
    let mut project_iter = stmt.query_map(params![path], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            last_cleaned: row
                .get::<_, Option<String>>(3)?
                .unwrap_or_else(|| "Never".to_string()),
            cache_dirs: Vec::new(),
        })
    })?;

    if let Some(project) = project_iter.next() {
        let mut project = project?;
        let cache_dirs = get_cache_directories_for_project(conn, project.id)?;
        project.cache_dirs = cache_dirs.iter().map(|cd| cd.path.clone()).collect();
        return Ok(Some(project));
    }
    Ok(None)
}

pub fn get_all_projects(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt =
        conn.prepare("SELECT id, name, path, last_cleaned FROM projects ORDER BY name")?;
    let project_iter = stmt.query_map([], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get(2)?,
            last_cleaned: row
                .get::<_, Option<String>>(3)?
                .unwrap_or_else(|| "Never".to_string()),
            cache_dirs: Vec::new(),
        })
    })?;

    let mut projects = Vec::new();
    for project in project_iter {
        let mut project = project?;
        let cache_dirs = get_cache_directories_for_project(conn, project.id)?;
        project.cache_dirs = cache_dirs.iter().map(|cd| cd.path.clone()).collect();
        projects.push(project);
    }
    Ok(projects)
}

pub fn update_project_last_cleaned(conn: &Connection, id: i32) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE projects SET last_cleaned = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    Ok(())
}

pub fn remove_project(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn project_exists(conn: &Connection, id: i32) -> Result<bool> {
    Ok(get_project_by_id(conn, id)?.is_some())
}

// Cache directory queries
pub fn add_cache_directory(conn: &Connection, project_id: i32, path: &str) -> Result<i32> {
    conn.execute(
        "INSERT OR IGNORE INTO cache_directories (project_id, path) VALUES (?1, ?2)",
        params![project_id, path],
    )?;

    // Get the ID of the inserted or existing record
    let id: i32 = conn.query_row(
        "SELECT id FROM cache_directories WHERE project_id = ?1 AND path = ?2",
        params![project_id, path],
        |row| row.get(0),
    )?;

    Ok(id)
}

pub fn get_cache_directories_for_project(
    conn: &Connection,
    project_id: i32,
) -> Result<Vec<CacheDirectory>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, path, size_bytes, last_checked 
         FROM cache_directories 
         WHERE project_id = ?1 
         ORDER BY path",
    )?;

    let cache_iter = stmt.query_map(params![project_id], |row| {
        Ok(CacheDirectory {
            id: row.get(0)?,
            project_id: row.get(1)?,
            path: row.get(2)?,
            size_bytes: row.get(3)?,
            last_checked: row.get(4)?,
        })
    })?;

    let mut cache_dirs = Vec::new();
    for cache_dir in cache_iter {
        cache_dirs.push(cache_dir?);
    }
    Ok(cache_dirs)
}

pub fn update_cache_directory_size(conn: &Connection, id: i32, size_bytes: i64) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE cache_directories SET size_bytes = ?1, last_checked = ?2 WHERE id = ?3",
        params![size_bytes, now, id],
    )?;
    Ok(())
}

pub fn remove_cache_directory(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM cache_directories WHERE id = ?1", params![id])?;
    Ok(())
}

// Cleaning history queries
pub fn add_cleaning_record(
    conn: &Connection,
    project_id: i32,
    size_freed: i64,
    cache_dirs_cleaned: i32,
) -> Result<i32> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO cleaning_history (project_id, cleaned_at, size_freed, cache_dirs_cleaned) 
         VALUES (?1, ?2, ?3, ?4)",
        params![project_id, now, size_freed, cache_dirs_cleaned],
    )?;
    Ok(conn.last_insert_rowid() as i32)
}

pub fn get_cleaning_history_for_project(
    conn: &Connection,
    project_id: i32,
) -> Result<Vec<CleaningHistory>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, cleaned_at, size_freed, cache_dirs_cleaned 
         FROM cleaning_history 
         WHERE project_id = ?1 
         ORDER BY cleaned_at DESC",
    )?;

    let history_iter = stmt.query_map(params![project_id], |row| {
        Ok(CleaningHistory {
            id: row.get(0)?,
            project_id: row.get(1)?,
            cleaned_at: row.get(2)?,
            size_freed: row.get(3)?,
            cache_dirs_cleaned: row.get(4)?,
        })
    })?;

    let mut history = Vec::new();
    for record in history_iter {
        history.push(record?);
    }
    Ok(history)
}

pub fn get_all_cleaning_history(conn: &Connection) -> Result<Vec<CleaningHistory>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, cleaned_at, size_freed, cache_dirs_cleaned 
         FROM cleaning_history 
         ORDER BY cleaned_at DESC",
    )?;

    let history_iter = stmt.query_map([], |row| {
        Ok(CleaningHistory {
            id: row.get(0)?,
            project_id: row.get(1)?,
            cleaned_at: row.get(2)?,
            size_freed: row.get(3)?,
            cache_dirs_cleaned: row.get(4)?,
        })
    })?;

    let mut history = Vec::new();
    for record in history_iter {
        history.push(record?);
    }
    Ok(history)
}

// Combined queries for complex operations
pub fn get_project_with_cache_dirs(
    conn: &Connection,
    project_id: i32,
) -> Result<Option<(Project, Vec<CacheDirectory>)>> {
    if let Some(project) = get_project_by_id(conn, project_id)? {
        let cache_dirs = get_cache_directories_for_project(conn, project_id)?;
        Ok(Some((project, cache_dirs)))
    } else {
        Ok(None)
    }
}

pub fn get_all_projects_with_cache_dirs(
    conn: &Connection,
) -> Result<Vec<(Project, Vec<CacheDirectory>)>> {
    let projects = get_all_projects(conn)?;
    let mut result = Vec::new();

    for project in projects {
        let cache_dirs = get_cache_directories_for_project(conn, project.id)?;
        result.push((project, cache_dirs));
    }

    Ok(result)
}

// Statistics queries
pub fn get_total_cache_size(conn: &Connection) -> Result<i64> {
    let size: Option<i64> = conn.query_row(
        "SELECT SUM(size_bytes) FROM cache_directories WHERE size_bytes IS NOT NULL",
        [],
        |row| row.get(0),
    )?;
    Ok(size.unwrap_or(0))
}

pub fn get_project_cache_size(conn: &Connection, project_id: i32) -> Result<i64> {
    let size: Option<i64> = conn.query_row(
        "SELECT SUM(size_bytes) FROM cache_directories WHERE project_id = ?1 AND size_bytes IS NOT NULL",
        params![project_id],
        |row| row.get(0),
    )?;
    Ok(size.unwrap_or(0))
}

pub fn get_total_size_freed(conn: &Connection) -> Result<i64> {
    let size: Option<i64> =
        conn.query_row("SELECT SUM(size_freed) FROM cleaning_history", [], |row| {
            row.get(0)
        })?;
    Ok(size.unwrap_or(0))
}

pub fn get_project_size_freed(conn: &Connection, project_id: i32) -> Result<i64> {
    let size: Option<i64> = conn.query_row(
        "SELECT SUM(size_freed) FROM cleaning_history WHERE project_id = ?1",
        params![project_id],
        |row| row.get(0),
    )?;
    Ok(size.unwrap_or(0))
}
