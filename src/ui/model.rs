use crate::commands::doctor::ProjectHealth;
use crate::core::DetectedProject;
use crate::db::Project;
use crate::utils::get_dir_size;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub path: String,
    pub size: u64,
}

/// One project row in the tree views: tracked (`id` set) or freshly scanned.
#[derive(Debug, Clone)]
pub struct Entry {
    pub id: Option<i32>,
    pub name: String,
    pub path: String,
    /// Ecosystem label, e.g. "Node.js + Rust" (scan results only).
    pub kind: String,
    pub last_cleaned: Option<String>,
    pub caches: Vec<CacheEntry>,
}

impl Entry {
    pub fn total_size(&self) -> u64 {
        self.caches.iter().map(|c| c.size).sum()
    }

    /// Cache path relative to the project root, for display.
    pub fn relative<'a>(&self, cache_path: &'a str) -> &'a str {
        cache_path
            .strip_prefix(&self.path)
            .map(|s| s.trim_start_matches(['/', '\\']))
            .filter(|s| !s.is_empty())
            .unwrap_or(cache_path)
    }

    pub fn matches(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        q.is_empty()
            || self.name.to_lowercase().contains(&q)
            || self.path.to_lowercase().contains(&q)
            || self.kind.to_lowercase().contains(&q)
            || self
                .caches
                .iter()
                .any(|c| c.path.to_lowercase().contains(&q))
    }

    pub fn from_projects(projects: &[Project]) -> Vec<Self> {
        projects
            .par_iter()
            .map(|p| Entry {
                id: Some(p.id),
                name: p.name.clone(),
                path: p.path.clone(),
                kind: String::new(),
                last_cleaned: p.last_cleaned.clone(),
                caches: sized(p.cache_dirs.iter().cloned()),
            })
            .collect()
    }

    pub fn from_detected(detected: &[DetectedProject]) -> Vec<Self> {
        detected
            .par_iter()
            .map(|p| Entry {
                id: None,
                name: p.name.clone(),
                path: p.path.to_string_lossy().into_owned(),
                kind: p.kind_label(),
                last_cleaned: None,
                caches: sized(
                    p.cache_dirs
                        .iter()
                        .map(|c| c.to_string_lossy().into_owned()),
                ),
            })
            .collect()
    }
}

fn sized(paths: impl Iterator<Item = String>) -> Vec<CacheEntry> {
    let paths: Vec<String> = paths.collect();
    paths
        .into_par_iter()
        .map(|path| CacheEntry {
            size: get_dir_size(&path),
            path,
        })
        .collect()
}

pub fn format_cleaned(raw: Option<&str>) -> String {
    match raw.map(chrono::DateTime::parse_from_rfc3339) {
        None => "Never".into(),
        Some(Ok(dt)) => dt
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M")
            .to_string(),
        Some(Err(_)) => raw.unwrap_or_default().chars().take(10).collect(),
    }
}

/// Replace the home directory prefix with `~`.
pub fn tilde(path: &str) -> String {
    match dirs::home_dir() {
        Some(home) => match path.strip_prefix(home.to_string_lossy().as_ref()) {
            Some(rest) if rest.is_empty() || rest.starts_with(['/', '\\']) => format!("~{}", rest),
            _ => path.to_string(),
        },
        None => path.to_string(),
    }
}

#[derive(Debug, Clone)]
pub struct UiDoctorItem {
    pub project_id: i32,
    pub project_name: String,
    pub project_path: String,
    pub path_exists: bool,
    pub cache_dirs: Vec<(String, bool, u64)>, // (path, exists, size)
    pub issues: Vec<String>,
    pub total_size: u64,
}

impl UiDoctorItem {
    pub fn from_healths(healths: &[ProjectHealth]) -> Vec<Self> {
        healths
            .par_iter()
            .map(|h| {
                let cache_dirs: Vec<(String, bool, u64)> = h
                    .cache_dirs_exist
                    .iter()
                    .map(|(dir, exists)| {
                        (
                            dir.clone(),
                            *exists,
                            if *exists { get_dir_size(dir) } else { 0 },
                        )
                    })
                    .collect();
                UiDoctorItem {
                    project_id: h.project_id,
                    project_name: h.project_name.clone(),
                    project_path: h.project_path.clone(),
                    path_exists: h.path_exists,
                    total_size: cache_dirs.iter().map(|(_, _, sz)| *sz).sum(),
                    cache_dirs,
                    issues: h.issues.clone(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_cleaned_handles_never_and_rfc3339() {
        assert_eq!(format_cleaned(None), "Never");
        assert_eq!(format_cleaned(Some("garbage-value")), "garbage-va");
        assert_eq!(format_cleaned(Some("2026-07-29T12:00:00Z")).len(), 16);
    }

    #[test]
    fn relative_cache_path() {
        let e = Entry {
            id: None,
            name: "app".into(),
            path: "/w/app".into(),
            kind: String::new(),
            last_cleaned: None,
            caches: vec![],
        };
        assert_eq!(e.relative("/w/app/src-tauri/target"), "src-tauri/target");
        assert_eq!(e.relative("/elsewhere/cache"), "/elsewhere/cache");
    }
}
