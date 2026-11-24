use crate::db;
use crate::utils::{get_dir_size, human_readable_size};
use colored::Colorize;

pub fn run() -> anyhow::Result<()> {
    let conn = db::connect()?;
    let projects = db::get_all_projects(&conn)?;
    let project_count = projects.len();
    let mut total_storage_occupied: u64 = 0;

    for project in projects {
        for cd in project.cache_dirs {
            total_storage_occupied += get_dir_size(&cd);
        }
    }

    println!();
    println!("{}", "Cache Statistics".bright_blue());
    println!("{:<18} {:>14}", "Projects tracked", project_count);
    println!("{:<18} {:>14}", "Total cache size", human_readable_size(total_storage_occupied));
    println!();

    Ok(())
}
