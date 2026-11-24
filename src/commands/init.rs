use crate::core::{ScanConfig, Scanner};
use crate::db;
use crate::utils::logger::{ask_input, info, success};
use anyhow::{Result, bail};
use colored::Colorize;
use std::path::PathBuf;

/// Track the current directory. Caches are auto-detected (including nested
/// sub-projects) unless `path` is given.
pub fn run(path: Option<PathBuf>) -> Result<()> {
    let project_dir = std::env::current_dir()?;
    let project_path = project_dir.to_string_lossy().to_string();
    let project_name = project_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| project_path.clone());

    if crate::core::is_container_dir(&project_dir, dirs::home_dir().as_deref()) {
        bail!("Refusing to track {} as a project; run init inside a project folder", project_path);
    }

    let conn = db::connect()?;
    if db::get_project_by_path(&conn, &project_path)?.is_some() {
        bail!("{} is already tracked", project_path);
    }

    let mut cache_dirs: Vec<PathBuf> = match path {
        Some(p) => vec![crate::utils::absolute(p)],
        None => {
            let config = crate::config::Config::load()?;
            let found = Scanner::new(ScanConfig::load()?, Vec::new())
                .scan(std::slice::from_ref(&project_dir), config.scan.max_depth);
            for p in &found {
                info(&format!("detected {} at {}", p.kind_label().bold(), p.path.display()));
            }
            found.into_iter().flat_map(|p| p.cache_dirs).collect()
        }
    };

    let prompt = if cache_dirs.is_empty() {
        "Cache directory (Enter to finish)"
    } else {
        "Extra cache directory (Enter to finish)"
    };
    loop {
        let input = ask_input(prompt);
        if input.is_empty() {
            break;
        }
        cache_dirs.push(crate::utils::absolute(input));
    }
    if cache_dirs.is_empty() {
        bail!("At least one cache directory must be provided");
    }

    let cache_paths: Vec<String> = cache_dirs.iter().map(|p| p.to_string_lossy().to_string()).collect();

    info(&format!("Project: {}", project_name.bold()));
    info(&format!("Path: {}", project_path.bold()));
    for cache in &cache_paths {
        info(&format!("  cache: {}", cache));
    }

    db::add_project(&conn, &project_name, &project_path, &cache_paths)?;
    success(&format!(
        "Tracking {} with {} cache director{}",
        project_name,
        cache_paths.len(),
        if cache_paths.len() == 1 { "y" } else { "ies" }
    ));
    Ok(())
}
