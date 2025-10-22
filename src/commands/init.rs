use crate::db::add_project;
use std::io;
use std::io::Write;
use std::path::PathBuf;

pub fn run(path: Option<PathBuf>) -> anyhow::Result<()> {
    let project_dir = std::env::current_dir().expect("Failed to get current directory");
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
                detected_cache = Some(project_dir.join(&cache));
                break;
            }
        }

        if let Some(cache) = detected_cache {
            cache
        } else {
            print!("Enter cache directory path :");
            io::stdout().flush()?;
            let mut input = String::new();

            io::stdin().read_line(&mut input)?;
            PathBuf::from(input.trim())
        }
    };

    // 2. Insert into DB
    let _ = add_project(
        &project_name,
        project_dir.to_str().unwrap(),
        cache_dir.to_str().unwrap(),
    );

    //  3. Confirm
    println!("New project created.");
    Ok(())
}
