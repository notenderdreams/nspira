use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub last_cleaned: String,
    pub cache_dirs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheDirectory {
    pub id: i32,
    pub project_id: i32,
    pub path: String,
    pub size_bytes: Option<i64>,
    pub last_checked: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleaningHistory {
    pub id: i32,
    pub project_id: i32,
    pub cleaned_at: String,
    pub size_freed: i64,
    pub cache_dirs_cleaned: i32,
}

pub const CREATE_PROJECTS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    last_cleaned TEXT
)
"#;

pub const CREATE_CACHE_DIRECTORIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS cache_directories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    size_bytes INTEGER,
    last_checked TEXT,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE,
    UNIQUE(project_id, path)
)
"#;

pub const CREATE_CLEANING_HISTORY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS cleaning_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    cleaned_at TEXT NOT NULL,
    size_freed INTEGER NOT NULL DEFAULT 0,
    cache_dirs_cleaned INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (project_id) REFERENCES projects (id) ON DELETE CASCADE
)
"#;

pub const CREATE_PROJECTS_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_projects_path ON projects(path)
"#;

pub const CREATE_CACHE_DIRS_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_cache_dirs_project_id ON cache_directories(project_id)
"#;

pub const CREATE_HISTORY_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_cleaning_history_project_id ON cleaning_history(project_id)
"#;

pub const CREATE_HISTORY_DATE_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS idx_cleaning_history_date ON cleaning_history(cleaned_at)
"#;
