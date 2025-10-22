use crate::db::add_project;
use std::path::{Path, PathBuf};

pub fn run(project_name: &str, cache_dir: PathBuf) -> anyhow::Result<()> {
    let project_dir = cache_dir.parent().unwrap_or_else(|| Path::new("."));

    let _ = add_project(
        project_name,
        project_dir.to_str().unwrap(),
        cache_dir.to_str().unwrap(),
    );

    println!("New project created.");
    Ok(())
}
