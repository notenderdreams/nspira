use crate::db::add_project;
use std::path::PathBuf;
use colored::Colorize;
use crate::utils::logger;
use crate::utils::logger::{info, success};

pub fn run(path: Option<PathBuf>) -> anyhow::Result<()> {
    let project_dir = std::env::current_dir()?;
    let project_name = project_dir
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let cache_dir = if let Some(p) = path {
        p
    } else {
        let identifier_patterns = vec![
            ("package.json", "node_modules"), // Node / JS
            ("Cargo.toml", "target"),         // Rust
        ];
        let mut detected_cache: Option<PathBuf> = None;
        for (iden, cache) in identifier_patterns {
            let iden_path = project_dir.join(iden);
            if iden_path.exists() {
                info(&format!("{} file found", iden.bold()));
                detected_cache = Some(project_dir.join(&cache));
                break;
            }
        }

        if let Some(cache) = detected_cache {
            info(&format!("{} has been selected as the cache directory", cache.to_str().unwrap().bold()));
            cache
        } else {
            PathBuf::from(logger::ask_input("Enter cache directory path"))
        }
    };

    // 2. Insert into DB
    let _ = add_project(
        &project_name,
        project_dir.to_str().unwrap(),
        cache_dir.to_str().unwrap(),
    )?;

    //  3. Confirm
    println!();
    success("New project created.");
    Ok(())
}
