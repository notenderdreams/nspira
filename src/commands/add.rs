use crate::db;
use crate::utils::logger::{ask_input, info, success};
use anyhow::{Result, bail};
use colored::Colorize;
use std::path::PathBuf;

pub fn run(project_name: &str, cache_dirs: Vec<PathBuf>) -> Result<()> {
    if cache_dirs.is_empty() {
        bail!("At least one cache directory must be provided");
    }

    let cwd = std::env::current_dir()?;
    let cache_dirs: Vec<PathBuf> = cache_dirs.into_iter().map(crate::utils::absolute).collect();

    // Project root = parent of the first cache dir.
    let project_path = cache_dirs[0]
        .parent()
        .unwrap_or(&cwd)
        .to_string_lossy()
        .to_string();
    let cache_paths: Vec<String> = cache_dirs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    info(&format!("Project: {}", project_name.bold()));
    info(&format!("Path: {}", project_path.bold()));
    for cache in &cache_paths {
        info(&format!("  cache: {}", cache));
    }

    if !ask_input("Add this project? (y/n)").eq_ignore_ascii_case("y") {
        info("Cancelled");
        return Ok(());
    }

    let conn = db::connect()?;
    if db::get_project_by_path(&conn, &project_path)?.is_some() {
        bail!("{} is already tracked", project_path);
    }
    db::add_project(&conn, project_name, &project_path, &cache_paths)?;

    success(&format!(
        "Tracking {} with {} cache director{}",
        project_name,
        cache_paths.len(),
        if cache_paths.len() == 1 { "y" } else { "ies" }
    ));
    Ok(())
}
