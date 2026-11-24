use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub path: String,
    /// RFC 3339 timestamp, or `None` if never cleaned.
    pub last_cleaned: Option<String>,
    pub cache_dirs: Vec<String>,
}

pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    last_cleaned TEXT
);

CREATE TABLE IF NOT EXISTS cache_directories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    size_bytes INTEGER,
    last_checked TEXT,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
    UNIQUE(project_id, path)
);

CREATE TABLE IF NOT EXISTS cleaning_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    cleaned_at TEXT NOT NULL,
    size_freed INTEGER NOT NULL DEFAULT 0,
    cache_dirs_cleaned INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS ignored_paths (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_cache_dirs_project_id ON cache_directories(project_id);
CREATE INDEX IF NOT EXISTS idx_cleaning_history_project_id ON cleaning_history(project_id);

-- Rows orphaned while foreign keys were not enforced.
DELETE FROM cache_directories WHERE project_id NOT IN (SELECT id FROM projects);
DELETE FROM cleaning_history WHERE project_id NOT IN (SELECT id FROM projects);
"#;
