use crate::db::Project;
use crate::utils::{get_dir_size, human_readable_size};
use chrono::{DateTime, NaiveDate};
use tabled::settings::Style;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct ProjectDisplay {
    id: i32,
    name: String,
    cache_path: String,
    size: String,
    last_cleaned: String,
}

pub fn format_date(date_str: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        dt.date().naive_local().to_string() // "YYYY-MM-DD"
    } else if let Ok(naive) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        naive.to_string()
    } else {
        date_str.to_string() // fallback if parsing fails
    }
}

pub fn print_projects(projects: Vec<Project>) {
    let display: Vec<ProjectDisplay> = projects
        .into_iter()
        .map(|p| ProjectDisplay {
            id: p.id,
            name: p.name,
            cache_path: p.cache_dir.clone(),
            size: human_readable_size(get_dir_size(&p.cache_dir)),
            last_cleaned: format_date(&p.last_cleaned),
        })
        .collect();

    let table = Table::new(display).with(Style::rounded()).to_string();
    println!("{}", table);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_print_projects() {
        // Create some fake directories/files for testing
        fs::create_dir_all("test_cache1").unwrap();
        fs::create_dir_all("test_cache2").unwrap();
        fs::write("test_cache1/file1.txt", b"Hello").unwrap();
        fs::write("test_cache2/file2.txt", b"World! Rust").unwrap();

        let projects = vec![
            Project {
                id: 1,
                name: "Nspira".to_string(),
                path: "/home/user/nspira".to_string(),
                cache_dir: "test_cache1".to_string(),
                last_cleaned: "2025-10-21".to_string(),
            },
            Project {
                id: 2,
                name: "VoidCrate".to_string(),
                path: "/home/user/voidcrate".to_string(),
                cache_dir: "test_cache2".to_string(),
                last_cleaned: "2025-10-20".to_string(),
            },
        ];

        print_projects(projects);

        // Cleanup
        fs::remove_dir_all("test_cache1").unwrap();
        fs::remove_dir_all("test_cache2").unwrap();
    }
}
