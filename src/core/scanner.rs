use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectPattern {
    pub name: String,
    pub identifier: String,
    pub cache_dirs: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScanConfig {
    pub patterns: Vec<ProjectPattern>,
    pub skip_dirs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DetectedProject {
    pub name: String,
    pub path: PathBuf,
    pub project_type: String,
    pub cache_dirs: Vec<PathBuf>,
}

impl ScanConfig {
    pub fn load() -> Result<Self> {
        // Try to load from user config first
        let config_path = dirs::config_dir().map(|d| d.join("nspira").join("patterns.json"));

        if let Some(path) = &config_path
            && path.exists()
        {
            let content = fs::read_to_string(path)?;
            return Ok(serde_json::from_str(&content)?);
        }

        // Fallback to embedded default
        Ok(serde_json::from_str(crate::DEFAULT_PATTERNS)?)
    }
}

pub struct Scanner {
    config: ScanConfig,
    tracked_paths: HashSet<PathBuf>,
}

impl Scanner {
    pub fn new(config: ScanConfig, tracked_paths: HashSet<PathBuf>) -> Self {
        Self {
            config,
            tracked_paths,
        }
    }

    pub fn scan(&self, start_path: &Path, max_depth: usize) -> Result<Vec<DetectedProject>> {
        let mut detected = Vec::new();
        let skip_set: HashSet<String> = self.config.skip_dirs.iter().cloned().collect();
        let mut scanned_count = 0;

        let walker = WalkDir::new(start_path)
            .max_depth(max_depth)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !skip_set.contains(name.as_ref())
            });

        for entry in walker.filter_map(|e| e.ok()) {
            if !entry.file_type().is_dir() {
                continue;
            }

            scanned_count += 1;
            if scanned_count % 100 == 0 {
                print!("\rScanning... {} directories checked", scanned_count);
                use std::io::{self, Write};
                io::stdout().flush()?;
            }

            let path = entry.path();

            // Skip if already tracked
            if self.tracked_paths.contains(path) {
                continue;
            }

            // Check if this directory matches any pattern
            if let Some(project) = self.detect_project(path) {
                detected.push(project);
            }
        }

        if scanned_count > 0 {
            println!("\rScanned {} directories", scanned_count);
        }

        Ok(detected)
    }

    fn detect_project(&self, path: &Path) -> Option<DetectedProject> {
        for pattern in &self.config.patterns {
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

                    return Some(DetectedProject {
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
}
