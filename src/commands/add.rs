use crate::db::add_project;
use crate::utils::logger::{success, ask_input, info};
use std::path::{Path, PathBuf};
use colored::Colorize;

pub fn run(project_name: &str, cache_dirs: Vec<PathBuf>) -> anyhow::Result<()> {
    if cache_dirs.is_empty() {
        anyhow::bail!("At least one cache directory must be provided");
    }

    // Convert all paths to absolute paths
    let cache_dirs: Vec<PathBuf> = cache_dirs
        .into_iter()
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(path)
            }
        })
        .collect();

    // Get project directory from the first cache dir's parent
    let project_dir = cache_dirs[0].parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();

    // Convert PathBuf vector to Vec<String> with absolute paths
    let cache_paths: Vec<String> = cache_dirs
        .iter()
        .map(|p| {
            if p.is_absolute() {
                p.to_string_lossy().to_string()
            } else {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join(p)
                    .to_string_lossy()
                    .to_string()
            }
        })
        .collect();

    // Convert project_dir to absolute path string
    let project_path = if project_dir.is_absolute() {
        project_dir.to_string_lossy().to_string()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(&project_dir)
            .to_string_lossy()
            .to_string()
    };

    // Summary
    info(&format!("Project: {}", project_name.bold()));
    info(&format!("Path: {}", project_path.bold()));
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

    match add_project(project_name, &project_path, cache_paths) {
        Ok(_) => {
            success(&format!("New project created with {} cache director{}",
                             cache_count,
                             if cache_count == 1 { "y" } else { "ies" }
            ));
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Failed to add project to database: {}", e);
        }
    }
}