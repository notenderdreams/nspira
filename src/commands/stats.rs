use crate::db::get_all_projects;
use crate::utils::{get_dir_size, human_readable_size};
use colored::Colorize;

pub fn run() -> anyhow::Result<()> {
    let projects = get_all_projects()?;
    let project_count = projects.len();
    let mut total_storage_occupied: u64 = 0;

    for project in projects {
        for cd in project.cache_dirs{
            total_storage_occupied += get_dir_size(&cd);
        }
    }

    println!();
    println!("╭─────────────────────────────────────╮");
    println!(
        "│  {}                   │",
        "Cache Statistics".bright_blue()
    );
    println!("├─────────────────────────────────────┤");
    println!("│  Projects tracked  │ {:>14} │", project_count);
    println!(
        "│  Total cache size  │ {:>14} │",
        human_readable_size(total_storage_occupied)
    );
    println!("╰─────────────────────────────────────╯");
    println!();

    Ok(())
}
