use crate::db::add_project;
use crate::utils::logger::{success, ask_input, info};
use std::path::{Path, PathBuf};
use colored::Colorize;

pub fn run(project_name: &str, cache_dirs: Vec<PathBuf>) -> anyhow::Result<()> {
    if cache_dirs.is_empty() {
        anyhow::bail!("At least one cache directory must be provided");
    }

    // Get project directory from the first cache dir's parent
    let project_dir = cache_dirs[0].parent().unwrap_or_else(|| Path::new("."));

    // Convert PathBuf vector to Vec<String>
    let cache_paths: Vec<String> = cache_dirs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    // Summery
    info(&format!("Project: {}", project_name.bold()));
    info(&format!("Path: {}", project_dir.to_str().unwrap().bold()));
    info(&format!("Cache directories ({}): ", cache_paths.len()));
    for (idx, cache) in cache_paths.iter().enumerate() {
        println!("  {}. {}", idx + 1, cache);
    }

    // Confirm
    let confirm = ask_input("Add this project? (y/n)");
    if confirm.to_lowercase() != "y" {
        info("Cancelled");
        return Ok(());
    }

    let cache_count = cache_paths.len();

    let _ = add_project(
        project_name,
        project_dir.to_str().unwrap(),
        cache_paths,
    )?;

    success(&format!("New project created with {} cache director{}",
                     cache_count,
                     if cache_count == 1 { "y" } else { "ies" }
    ));
    Ok(())
}