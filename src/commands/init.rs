use crate::db::add_project;
use crate::utils::logger;
use crate::utils::logger::{info, success};
use colored::Colorize;
use std::path::PathBuf;

pub fn run(path: Option<PathBuf>) -> anyhow::Result<()> {
    let project_dir = std::env::current_dir()?;
    let project_name = project_dir
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let cache_dirs = if let Some(p) = path {
        vec![p]
    } else {
        let identifier_patterns = vec![
            (
                "package.json",
                vec!["node_modules", ".next", "dist", "build"],
            ), // Node / JS
            ("Cargo.toml", vec!["target"]),             // Rust
            ("pom.xml", vec!["target"]),                // Maven
            ("build.gradle", vec!["build", ".gradle"]), // Gradle
            ("go.mod", vec!["bin", "pkg"]),             // Go
            ("requirements.txt", vec!["__pycache__", ".venv", "venv"]), // Python
        ];

        let mut detected_caches: Vec<PathBuf> = Vec::new();

        for (identifier, possible_caches) in identifier_patterns {
            let iden_path = project_dir.join(identifier);
            if iden_path.exists() {
                info(&format!("{} file found", identifier.bold()));

                // Check which cache directories actually exist
                for cache in possible_caches {
                    let cache_path = project_dir.join(cache);
                    if cache_path.exists() {
                        detected_caches.push(cache_path);
                        info(&format!("  ✓ Found cache directory: {}", cache.bold()));
                    }
                }
                break;
            }
        }

        if detected_caches.is_empty() {
            //  to provide cache directories manually
            let mut manual_caches = Vec::new();
            loop {
                let cache_input =
                    logger::ask_input("Enter cache directory path (or press Enter to finish)");
                if cache_input.trim().is_empty() {
                    break;
                }
                manual_caches.push(PathBuf::from(cache_input));
            }

            if manual_caches.is_empty() {
                anyhow::bail!("At least one cache directory must be provided");
            }
            manual_caches
        } else {
            // to add more cache directories
            let add_more = logger::ask_input("Add more cache directories? (y/n)");
            if add_more.to_lowercase() == "y" {
                loop {
                    let cache_input =
                        logger::ask_input("Enter cache directory path (or press Enter to finish)");
                    if cache_input.trim().is_empty() {
                        break;
                    }
                    detected_caches.push(PathBuf::from(cache_input));
                }
            }
            detected_caches
        }
    };

    // Display summary
    println!();
    info(&format!("Project: {}", project_name.bold()));
    info(&format!("Path: {}", project_dir.to_str().unwrap().bold()));
    info(&format!("Cache directories ({}): ", cache_dirs.len()));
    for (idx, cache) in cache_dirs.iter().enumerate() {
        println!("  {}. {}", idx + 1, cache.to_str().unwrap());
    }

    // Convert to Vec<String>
    let cache_paths: Vec<String> = cache_dirs
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    // Add project
    let _ = add_project(&project_name, project_dir.to_str().unwrap(), cache_paths)?;

    // Confirm
    println!();
    success(&format!(
        "New project created with {} cache director{}",
        cache_dirs.len(),
        if cache_dirs.len() == 1 { "y" } else { "ies" }
    ));
    Ok(())
}
