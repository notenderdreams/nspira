use anyhow::Result;
use chrono::Utc;
use rusqlite::{Connection, OptionalExtension, Row, params};

use super::schema::Project;

fn project_from_row(row: &Row) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        last_cleaned: row.get(3)?,
        cache_dirs: Vec::new(),
    })
}

fn with_cache_dirs(conn: &Connection, mut project: Project) -> Result<Project> {
    project.cache_dirs = get_cache_dirs(conn, project.id)?;
    Ok(project)
}

// Projects

/// Insert a project and its cache dirs in one transaction. Returns the new id.
pub fn add_project(conn: &Connection, name: &str, path: &str, cache_dirs: &[String]) -> Result<i32> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("INSERT INTO projects (name, path) VALUES (?1, ?2)", params![name, path])?;
    let id = tx.last_insert_rowid() as i32;
    for dir in cache_dirs {
        tx.execute(
            "INSERT OR IGNORE INTO cache_directories (project_id, path) VALUES (?1, ?2)",
            params![id, dir],
        )?;
    }
    tx.commit()?;
    Ok(id)
}

pub fn get_project_by_id(conn: &Connection, id: i32) -> Result<Option<Project>> {
    let project = conn
        .query_row(
            "SELECT id, name, path, last_cleaned FROM projects WHERE id = ?1",
            params![id],
            project_from_row,
        )
        .optional()?;
    project.map(|p| with_cache_dirs(conn, p)).transpose()
}

pub fn get_project_by_path(conn: &Connection, path: &str) -> Result<Option<Project>> {
    let project = conn
        .query_row(
            "SELECT id, name, path, last_cleaned FROM projects WHERE path = ?1",
            params![path],
            project_from_row,
        )
        .optional()?;
    project.map(|p| with_cache_dirs(conn, p)).transpose()
}

pub fn get_all_projects(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare("SELECT id, name, path, last_cleaned FROM projects ORDER BY name")?;
    let projects = stmt.query_map([], project_from_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
    projects.into_iter().map(|p| with_cache_dirs(conn, p)).collect()
}

pub fn remove_project(conn: &Connection, id: i32) -> Result<bool> {
    Ok(conn.execute("DELETE FROM projects WHERE id = ?1", params![id])? > 0)
}

// Cache directories

pub fn get_cache_dirs(conn: &Connection, project_id: i32) -> Result<Vec<String>> {
    let mut stmt =
        conn.prepare("SELECT path FROM cache_directories WHERE project_id = ?1 ORDER BY path")?;
    let dirs = stmt
        .query_map(params![project_id], |r| r.get(0))?
        .collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(dirs)
}

pub fn remove_cache_dir(conn: &Connection, project_id: i32, path: &str) -> Result<bool> {
    Ok(conn.execute(
        "DELETE FROM cache_directories WHERE project_id = ?1 AND path = ?2",
        params![project_id, path],
    )? > 0)
}

// Cleaning

/// Record a clean run and bump the project's last-cleaned timestamp.
pub fn record_clean(conn: &Connection, project_id: i32, bytes_freed: u64, dirs_cleaned: usize) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT INTO cleaning_history (project_id, cleaned_at, size_freed, cache_dirs_cleaned)
         VALUES (?1, ?2, ?3, ?4)",
        params![project_id, now, bytes_freed as i64, dirs_cleaned as i64],
    )?;
    tx.execute("UPDATE projects SET last_cleaned = ?1 WHERE id = ?2", params![now, project_id])?;
    tx.commit()?;
    Ok(())
}

pub fn total_bytes_freed(conn: &Connection) -> Result<u64> {
    let total: Option<i64> =
        conn.query_row("SELECT SUM(size_freed) FROM cleaning_history", [], |r| r.get(0))?;
    Ok(total.unwrap_or(0).max(0) as u64)
}

// Ignored paths

pub fn add_ignored(conn: &Connection, path: &str) -> Result<bool> {
    Ok(conn.execute(
        "INSERT OR IGNORE INTO ignored_paths (path, created_at) VALUES (?1, ?2)",
        params![path, Utc::now().to_rfc3339()],
    )? > 0)
}

pub fn remove_ignored(conn: &Connection, path: &str) -> Result<bool> {
    Ok(conn.execute("DELETE FROM ignored_paths WHERE path = ?1", params![path])? > 0)
}

pub fn get_ignored(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT path FROM ignored_paths ORDER BY path")?;
    let paths = stmt.query_map([], |r| r.get(0))?.collect::<rusqlite::Result<Vec<String>>>()?;
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        super::super::prepare(&conn).unwrap();
        conn
    }

    #[test]
    fn remove_project_cascades_to_cache_dirs() {
        let conn = mem();
        let id = add_project(&conn, "p", "/p", &["/p/target".into()]).unwrap();
        assert_eq!(get_cache_dirs(&conn, id).unwrap().len(), 1);
        assert!(remove_project(&conn, id).unwrap());
        let n: i64 = conn
            .query_row("SELECT COUNT(*) FROM cache_directories", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn record_clean_sets_last_cleaned_and_history() {
        let conn = mem();
        let id = add_project(&conn, "p", "/p", &[]).unwrap();
        record_clean(&conn, id, 100, 2).unwrap();
        assert!(get_project_by_id(&conn, id).unwrap().unwrap().last_cleaned.is_some());
        assert_eq!(total_bytes_freed(&conn).unwrap(), 100);
    }

    #[test]
    fn ignored_paths_roundtrip() {
        let conn = mem();
        assert!(add_ignored(&conn, "/x").unwrap());
        assert!(!add_ignored(&conn, "/x").unwrap());
        assert_eq!(get_ignored(&conn).unwrap(), vec!["/x".to_string()]);
        assert!(remove_ignored(&conn, "/x").unwrap());
        assert!(get_ignored(&conn).unwrap().is_empty());
    }
}
