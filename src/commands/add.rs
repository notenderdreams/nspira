use crate::db::add_project;
use crate::utils::logger::success;
use std::path::{Path, PathBuf};

pub fn run(project_name: &str, cache_dir: PathBuf) -> anyhow::Result<()> {
    let project_dir = cache_dir.parent().unwrap_or_else(|| Path::new("."));

    let _ = add_project(
        project_name,
        project_dir.to_str().unwrap(),
        cache_dir.to_str().unwrap(),
    )?;

    success("New project created.");
    Ok(())
}
