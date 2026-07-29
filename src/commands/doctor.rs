use crate::db;
use crate::ui::views::run_doctor_view;
use crate::utils::logger::{info, task};
use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ProjectHealth {
    pub project_id: i32,
    pub project_name: String,
    pub project_path: String,
    pub path_exists: bool,
    pub cache_dirs_exist: Vec<(String, bool)>,
    pub issues: Vec<String>,
}

pub fn run() -> Result<()> {
    task("Running health check...");

    let conn = db::connect()?;
    let projects = db::get_all_projects(&conn)?;

    if projects.is_empty() {
        info("No projects found in database.");
        return Ok(());
    }

    info(&format!("Checking {} tracked projects...", projects.len()));

    let mut project_healths = Vec::new();
    let mut healthy_count = 0;
    let mut total_issues = 0;

    for project in &projects {
        let mut health = ProjectHealth {
            project_id: project.id,
            project_name: project.name.clone(),
            project_path: project.path.clone(),
            path_exists: false,
            cache_dirs_exist: Vec::new(),
            issues: Vec::new(),
        };

        // Check if project path exists
        health.path_exists = Path::new(&project.path).exists();
        if !health.path_exists {
            health
                .issues
                .push(format!("Project path does not exist: {}", project.path));
        }

        // Check each cache directory
        for cache_dir in &project.cache_dirs {
            let exists = Path::new(cache_dir).exists();
            health.cache_dirs_exist.push((cache_dir.clone(), exists));
            if !exists {
                health
                    .issues
                    .push(format!("Cache directory does not exist: {}", cache_dir));
            }
        }

        if health.issues.is_empty() {
            healthy_count += 1;
        }
        total_issues += health.issues.len();

        project_healths.push(health);
    }

    // Launch doctor TUI
    run_doctor_view(project_healths, healthy_count, total_issues)?;

    Ok(())
}
