use crate::db::get_all_projects;
use crate::utils::{get_dir_size, human_readable_size};

pub fn run() -> anyhow::Result<()> {
    let projects = get_all_projects()?;
    let project_count = projects.len();
    let mut total_storage_occupied: u64 = 0;

    for project in projects {
        total_storage_occupied += get_dir_size(&project.cache_dir);
    }

    println!("Number of projects----:        {}", project_count);
    println!(
        "Total storage occupied: {}",
        human_readable_size(total_storage_occupied)
    );

    Ok(())
}
