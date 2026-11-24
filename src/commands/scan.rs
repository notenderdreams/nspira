use crate::core::{ScanConfig, Scanner};
use crate::utils::logger::{info, success, task};
use anyhow::Result;
use std::path::PathBuf;

pub fn run(custom_path: Option<PathBuf>) -> Result<()> {
    let app_config = crate::config::Config::load()?;
    crate::utils::parallel::configure_thread_pool(app_config.scan.parallelism)?;

    let mut scan_config = ScanConfig::load()?;
    for dir in &app_config.scan.skip_directories {
        if !scan_config.skip_dirs.contains(dir) {
            scan_config.skip_dirs.push(dir.clone());
        }
    }

    let roots = match custom_path {
        Some(p) => vec![crate::utils::absolute(p)],
        None => crate::utils::fs::get_default_scan_paths(),
    };

    // Tracked projects and ignored paths are pruned with their whole subtree.
    let conn = crate::db::connect()?;
    let ignored = crate::db::get_ignored(&conn)?;
    let mut excluded: Vec<PathBuf> = crate::db::get_all_projects(&conn)?
        .into_iter()
        .map(|p| PathBuf::from(p.path))
        .collect();
    excluded.extend(ignored.iter().map(PathBuf::from));

    for root in &roots {
        task(&format!("Scanning {}", root.display()));
    }
    info(&format!(
        "{} patterns, depth {}, {} ignored path(s)",
        scan_config.patterns.len(),
        app_config.scan.max_depth,
        ignored.len()
    ));

    let detected = Scanner::new(scan_config, excluded).scan(&roots, app_config.scan.max_depth);
    if detected.is_empty() {
        info("No untracked projects found");
        return Ok(());
    }
    success(&format!("Found {} untracked project(s)", detected.len()));

    crate::ui::views::run_scan_view(detected)
}
