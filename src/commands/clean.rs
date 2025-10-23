use anyhow::Result;
use crate::utils::logger::{ success, task};

pub fn run(id: Option<i32>) -> Result<()> {
    match id {
        Some(pid) => {
            task(&format!("Cleaning cache for project ID {}", pid));
            clean_cache(pid)?;
            success("Cleaned!");
        }
        None => {
            task("Cleaning All Projects");
            let projects = crate::db::get_all_projects()?;
            for project in projects {
                clean_cache(project.id)?;
            }
            success("Cleaned!");
        }
    }
    Ok(())
}

fn clean_cache(pid: i32) -> Result<()> {
    let project = crate::db::get_project_by_id(pid)?.expect("No such project found with that id");

    crate::utils::clean_dir(&project.cache_dir)?;
    crate::db::update_last_cleaned(pid)?;
    Ok(())
}
