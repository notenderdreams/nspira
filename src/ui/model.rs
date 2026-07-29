use crate::commands::doctor::ProjectHealth;
use crate::core::DetectedProject;
use crate::db::Project;
use crate::utils::get_dir_size;
use rayon::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct UiCacheItem {
    pub path: String,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct UiProjectItem {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub cache_dirs: Vec<UiCacheItem>,
    pub total_size: u64,
    pub last_cleaned: String,
    pub formatted_last_cleaned: String,
}

impl UiProjectItem {
    pub fn from_projects(projects: &[Project]) -> Vec<Self> {
        projects
            .par_iter()
            .map(|p| {
                let cache_dirs: Vec<UiCacheItem> = p
                    .cache_dirs
                    .par_iter()
                    .map(|cd| {
                        let sz = get_dir_size(cd);
                        UiCacheItem {
                            path: cd.clone(),
                            size: sz,
                        }
                    })
                    .collect();

                let total_size: u64 = cache_dirs.iter().map(|c| c.size).sum();
                let formatted_last_cleaned = format_cleaned_date(&p.last_cleaned);

                UiProjectItem {
                    id: p.id,
                    name: p.name.clone(),
                    path: p.path.clone(),
                    cache_dirs,
                    total_size,
                    last_cleaned: p.last_cleaned.clone(),
                    formatted_last_cleaned,
                }
            })
            .collect()
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
                    .par_iter()
                    .map(|(dir, exists)| {
                        let sz = if *exists { get_dir_size(dir) } else { 0 };
                        (dir.clone(), *exists, sz)
                    })
                    .collect();

                let total_size: u64 = cache_dirs.iter().map(|(_, _, sz)| *sz).sum();

                UiDoctorItem {
                    project_id: h.project_id,
                    project_name: h.project_name.clone(),
                    project_path: h.project_path.clone(),
                    path_exists: h.path_exists,
                    cache_dirs,
                    issues: h.issues.clone(),
                    total_size,
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct UiScanItem {
    pub name: String,
    pub project_type: String,
    pub path: PathBuf,
    pub cache_dirs: Vec<(PathBuf, u64)>,
    pub total_size: u64,
}

impl UiScanItem {
    pub fn from_detected(detected: &[DetectedProject]) -> Vec<Self> {
        detected
            .par_iter()
            .map(|p| {
                let cache_dirs: Vec<(PathBuf, u64)> = p
                    .cache_dirs
                    .par_iter()
                    .map(|cd| {
                        let sz = get_dir_size(cd.to_str().unwrap_or(""));
                        (cd.clone(), sz)
                    })
                    .collect();

                let total_size: u64 = cache_dirs.iter().map(|(_, sz)| *sz).sum();

                UiScanItem {
                    name: p.name.clone(),
                    project_type: p.project_type.clone(),
                    path: p.path.clone(),
                    cache_dirs,
                    total_size,
                }
            })
            .collect()
    }
}

fn format_cleaned_date(raw_date: &str) -> String {
    if raw_date == "Never" || raw_date.is_empty() {
        "Never".to_string()
    } else if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(raw_date) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        raw_date.chars().take(10).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_cleaned_date() {
        assert_eq!(format_cleaned_date("Never"), "Never");
        assert_eq!(format_cleaned_date(""), "Never");
        assert_eq!(format_cleaned_date("2026-07-29T12:00:00Z"), "2026-07-29 12:00");
    }

    #[test]
    fn test_ui_project_item_conversion() {
        let p = Project {
            id: 1,
            name: "test_proj".to_string(),
            path: "/path/to/test".to_string(),
            last_cleaned: "Never".to_string(),
            cache_dirs: vec![],
        };
        let items = UiProjectItem::from_projects(&[p]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, 1);
        assert_eq!(items[0].name, "test_proj");
        assert_eq!(items[0].formatted_last_cleaned, "Never");
        assert_eq!(items[0].total_size, 0);
    }
}
