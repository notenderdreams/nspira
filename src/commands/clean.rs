use crate::core::{CleanTotals, clean_project};
use crate::utils::human_readable_size;
use crate::utils::logger::{info, success, task, warn};
use anyhow::{Result, anyhow};

pub fn run(id: Option<i32>) -> Result<()> {
    let conn = crate::db::connect()?;

    let projects = match id {
        Some(pid) => vec![
            crate::db::get_project_by_id(&conn, pid)?
                .ok_or_else(|| anyhow!("No project found with id {}", pid))?,
        ],
        None => crate::db::get_all_projects(&conn)?,
    };
    if projects.is_empty() {
        info("No tracked projects");
        return Ok(());
    }

    let mut totals = CleanTotals::default();
    for project in &projects {
        // Header only for projects that actually have something to clean.
        let mut header_shown = false;
        let t = clean_project(
            &conn,
            project.id,
            &project.cache_dirs,
            |path, result| match result {
                Ok(0) => {}
                Ok(bytes) => {
                    if !header_shown {
                        task(&format!("{} (id {})", project.name, project.id));
                        header_shown = true;
                    }
                    success(&format!(
                        "cleaned {} ({})",
                        path,
                        human_readable_size(*bytes)
                    ));
                }
                Err(e) => warn(&format!("failed {}: {}", path, e)),
            },
        )?;
        totals.add(t);
    }

    if totals.dirs == 0 && totals.failed == 0 {
        info("Nothing to clean");
    } else {
        success(&format!(
            "Cleaned {} {}, {} freed",
            totals.dirs,
            if totals.dirs == 1 {
                "directory"
            } else {
                "directories"
            },
            human_readable_size(totals.bytes)
        ));
    }
    Ok(())
}
