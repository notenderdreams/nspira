use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Parallel directory size calculation
pub fn calculate_sizes_parallel(paths: &[String]) -> Vec<(String, u64)> {
    let progress = ProgressBar::new(paths.len() as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    progress.set_message("Calculating sizes...");

    let results: Vec<(String, u64)> = paths
        .par_iter()
        .map(|path| {
            let size = crate::utils::get_dir_size(path);
            progress.inc(1);
            (path.clone(), size)
        })
        .collect();

    progress.finish_with_message("Size calculation complete");
    results
}

/// Parallel cache cleaning with progress
pub fn clean_caches_parallel(cache_dirs: &[String]) -> Result<u64> {
    let progress = ProgressBar::new(cache_dirs.len() as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    progress.set_message("Cleaning caches...");

    let total_freed = Arc::new(AtomicU64::new(0));
    let errors: Vec<Result<u64>> = cache_dirs
        .par_iter()
        .map(|cache_dir| {
            let size_before = crate::utils::get_dir_size(cache_dir);

            match crate::utils::clean_dir(cache_dir) {
                Ok(_) => {
                    let freed = size_before; // Assume all was freed
                    total_freed.fetch_add(freed, Ordering::Relaxed);
                    progress.inc(1);
                    Ok(freed)
                }
                Err(e) => {
                    progress.inc(1);
                    Err(e)
                }
            }
        })
        .collect();

    progress.finish_with_message("Cache cleaning complete");

    // Check for errors
    let error_count = errors.iter().filter(|r| r.is_err()).count();
    if error_count > 0 {
        eprintln!("Warning: {} cache directories failed to clean", error_count);
    }

    Ok(total_freed.load(Ordering::Relaxed))
}

/// Parallel filesystem scanning
pub fn scan_directories_parallel(
    directories: Vec<std::path::PathBuf>,
    max_depth: usize,
    skip_dirs: &[String],
) -> Vec<std::path::PathBuf> {
    let progress = ProgressBar::new(directories.len() as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    progress.set_message("Scanning directories...");

    let skip_set: std::collections::HashSet<String> = skip_dirs.iter().cloned().collect();

    let found_dirs: Vec<std::path::PathBuf> = directories
        .par_iter()
        .flat_map(|dir| {
            let mut results = Vec::new();

            if let Ok(walker) = walkdir::WalkDir::new(dir)
                .max_depth(max_depth)
                .follow_links(false)
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
            {
                for entry in walker {
                    if entry.file_type().is_dir() {
                        let name = entry.file_name().to_string_lossy();
                        if !skip_set.contains(name.as_ref()) {
                            results.push(entry.path().to_path_buf());
                        }
                    }
                }
            }

            progress.inc(1);
            results
        })
        .collect();

    progress.finish_with_message("Directory scanning complete");
    found_dirs
}

/// Parallel project detection
pub fn detect_projects_parallel(
    directories: &[std::path::PathBuf],
    patterns: &[crate::core::ProjectPattern],
) -> Vec<crate::core::DetectedProject> {
    let progress = ProgressBar::new(directories.len() as u64);
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    progress.set_message("Detecting projects...");

    let detected: Vec<crate::core::DetectedProject> = directories
        .par_iter()
        .filter_map(|path| {
            let result = detect_project_at_path(path, patterns);
            progress.inc(1);
            result
        })
        .collect();

    progress.finish_with_message("Project detection complete");
    detected
}

fn detect_project_at_path(
    path: &Path,
    patterns: &[crate::core::ProjectPattern],
) -> Option<crate::core::DetectedProject> {
    for pattern in patterns {
        let identifier_path = path.join(&pattern.identifier);

        if identifier_path.exists() {
            // Find which cache directories actually exist
            let mut found_caches = Vec::new();

            for cache_dir in &pattern.cache_dirs {
                let cache_path = path.join(cache_dir);
                if cache_path.exists() && cache_path.is_dir() {
                    found_caches.push(cache_path);
                }
            }

            // Only return if we found at least one cache directory
            if !found_caches.is_empty() {
                let project_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                return Some(crate::core::DetectedProject {
                    name: project_name,
                    path: path.to_path_buf(),
                    project_type: pattern.name.clone(),
                    cache_dirs: found_caches,
                });
            }
        }
    }

    None
}

/// Configure rayon thread pool based on user config
pub fn configure_thread_pool(parallelism: usize) -> Result<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(parallelism)
        .build_global()
        .map_err(|e| anyhow::anyhow!("Failed to configure thread pool: {}", e))?;

    Ok(())
}
