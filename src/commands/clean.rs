use crate::utils::logger::{success, task};
use crate::utils::{get_dir_size, human_readable_size};
use anyhow::Result;

pub fn run(id: Option<i32>) -> Result<()> {
    let conn = crate::db::connect()?;

    match id {
        Some(pid) => {
            task(&format!("Cleaning cache for project ID {}", pid));
            let size = clean_cache(&conn, pid)?;
            success(&format!(
                "{} freed from the disk",
                human_readable_size(size)
            ));
        }
        None => {
            task("Cleaning all caches");
            let projects = crate::db::get_all_projects(&conn)?;
            let mut size = 0;
            for project in projects {
                size += clean_cache(&conn, project.id)?;
            }
            success(&format!(
                "{} freed from the disk",
                human_readable_size(size)
            ));
        }
    }
    Ok(())
}

fn clean_cache(conn: &rusqlite::Connection, pid: i32) -> Result<u64> {
    let project =
        crate::db::get_project_by_id(conn, pid)?.expect("No such project found with that id");
    let mut size = 0;
    for cd in project.cache_dirs {
        size += get_dir_size(&cd);
        crate::utils::clean_dir(&cd)?;
    }
    crate::db::update_project_last_cleaned(conn, pid)?;
    Ok(size)
}
