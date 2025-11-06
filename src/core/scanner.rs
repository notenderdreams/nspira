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
        println!("🚀 Starting parallel scan...");

        // First, collect all directories to scan
        let mut all_dirs = Vec::new();
        let skip_set: HashSet<String> = self.config.skip_dirs.iter().cloned().collect();

        let walker = WalkDir::new(start_path)
            .max_depth(max_depth)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !skip_set.contains(name.as_ref())
            });

        for entry in walker.filter_map(|e| e.ok()) {
            if entry.file_type().is_dir() {
                let path = entry.path();

                // Skip if already tracked
                if !self.tracked_paths.contains(path) {
                    all_dirs.push(path.to_path_buf());
                }
            }
        }

        println!("📁 Found {} directories to scan", all_dirs.len());

        // Use parallel project detection
        let detected =
            crate::utils::parallel::detect_projects_parallel(&all_dirs, &self.config.patterns);

        println!("✅ Detected {} projects", detected.len());
        Ok(detected)
    }
}
