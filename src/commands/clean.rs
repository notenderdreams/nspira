use crate::utils::logger::{info, success, task, warn};
use crate::utils::{get_dir_size, human_readable_size};
use anyhow::{Result, anyhow};
use std::path::Path;

#[derive(Default)]
struct Totals {
    dirs: usize,
    bytes: u64,
}

pub fn run(id: Option<i32>) -> Result<()> {
    let conn = crate::db::connect()?;
    let mut totals = Totals::default();

    match id {
        Some(pid) => clean_project(&conn, pid, &mut totals)?,
        None => {
            let projects = crate::db::get_all_projects(&conn)?;
            if projects.is_empty() {
                info("No tracked projects");
                return Ok(());
            }
            for project in projects {
                clean_project(&conn, project.id, &mut totals)?;
            }
        }
    }

    success(&format!(
        "Cleaned {} {}, {} freed",
        totals.dirs,
        if totals.dirs == 1 { "directory" } else { "directories" },
        human_readable_size(totals.bytes)
    ));
    Ok(())
}

fn clean_project(conn: &rusqlite::Connection, pid: i32, totals: &mut Totals) -> Result<()> {
    let project = crate::db::get_project_by_id(conn, pid)?
        .ok_or_else(|| anyhow!("No project found with id {}", pid))?;

    task(&format!("{} (id {})", project.name, project.id));

    for cd in project.cache_dirs {
        if !Path::new(&cd).exists() {
            warn(&format!("skipped {} (not found)", cd));
            continue;
        }
        let size = get_dir_size(&cd);
        match crate::utils::clean_dir(&cd) {
            Ok(_) => {
                success(&format!("cleaned {} ({})", cd, human_readable_size(size)));
                totals.dirs += 1;
                totals.bytes += size;
            }
            Err(e) => warn(&format!("failed {}: {}", cd, e)),
        }
    }

    crate::db::update_project_last_cleaned(conn, pid)?;
    Ok(())
}
