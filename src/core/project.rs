use crate::db::{self, Project};
use anyhow::Result;

/// Project domain logic
pub struct ProjectManager;

impl ProjectManager {
    /// Get all tracked projects
    pub fn get_all() -> Result<Vec<Project>> {
        let conn = db::connect()?;
        db::get_all_projects(&conn)
    }

    /// Get a project by ID
    pub fn get_by_id(id: i32) -> Result<Option<Project>> {
        let conn = db::connect()?;
        db::get_project_by_id(&conn, id)
    }

    /// Add a new project
    pub fn add(name: &str, path: &str, cache_dirs: Vec<String>) -> Result<i32> {
        let conn = db::connect()?;
        let project_id = db::add_project(&conn, name, path)?;

        // Add cache directories
        for cache_dir in cache_dirs {
            db::add_cache_directory(&conn, project_id, &cache_dir)?;
        }

        Ok(project_id)
    }

    /// Remove a project
    pub fn remove(id: i32) -> Result<()> {
        let conn = db::connect()?;
        db::remove_project(&conn, id)
    }

    /// Check if a project exists
    pub fn exists(id: i32) -> Result<bool> {
        let conn = db::connect()?;
        db::project_exists(&conn, id)
    }

    /// Update last cleaned timestamp
    pub fn update_last_cleaned(id: i32) -> Result<()> {
        let conn = db::connect()?;
        db::update_project_last_cleaned(&conn, id)
    }
}
