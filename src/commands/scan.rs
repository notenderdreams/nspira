use crate::core::{ScanConfig, Scanner};
use crate::ui::views::run_scan_view;
use crate::utils::logger::{info, task};
use anyhow::Result;
use std::collections::HashSet;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    task("Loading configuration...");
    let app_config = crate::config::Config::load()?;

    // Configure thread pool for parallel operations
    crate::utils::parallel::configure_thread_pool(app_config.scan.parallelism)?;

    task("Loading scan patterns...");
    let mut scan_config = ScanConfig::load()?;

    // Merge skip directories from app config
    for skip_dir in &app_config.scan.skip_directories {
        if !scan_config.skip_dirs.contains(skip_dir) {
            scan_config.skip_dirs.push(skip_dir.clone());
        }
    }

    info(&format!(
        "Loaded {} project patterns",
        scan_config.patterns.len()
    ));

    task("Starting filesystem scan...");
    let start_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    info(&format!("Scanning from: {}", start_path.display()));
    info(&format!("Max depth: {}", app_config.scan.max_depth));
    println!();

    let tracked_paths = get_tracked_paths()?;
    let scanner = Scanner::new(scan_config, tracked_paths);
    let detected = scanner.scan(&start_path, app_config.scan.max_depth)?;

    // Run interactive TUI view
    run_scan_view(detected)?;

    Ok(())
}

fn get_tracked_paths() -> Result<HashSet<PathBuf>> {
    let projects = crate::core::ProjectManager::get_all()?;
    Ok(projects
        .into_iter()
        .map(|p| PathBuf::from(p.path))
        .collect())
}
