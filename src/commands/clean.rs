use crate::utils::logger::{success, task};
use crate::utils::{get_dir_size, human_readable_size};
use anyhow::Result;

pub fn run(id: Option<i32>) -> Result<()> {
    match id {
        Some(pid) => {
            task(&format!("Cleaning cache for project ID {}", pid));
            let size = clean_cache(pid)?;
            success(&format!(
                "{} freed from the disk",
                human_readable_size(size)
            ));
        }
        None => {
            task("Cleaning all caches");
            let projects = crate::db::get_all_projects()?;
            let mut size = 0;
            for project in projects {
                size += clean_cache(project.id)?;
            }
            success(&format!(
                "{} freed from the disk",
                human_readable_size(size)
            ));
        }
    }
    Ok(())
}

fn clean_cache(pid: i32) -> Result<u64> {
    let project = crate::db::get_project_by_id(pid)?.expect("No such project found with that id");
    let size = get_dir_size(&project.cache_dir);
    crate::utils::clean_dir(&project.cache_dir)?;
    crate::db::update_last_cleaned(pid)?;
    Ok(size)
}
