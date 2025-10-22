use anyhow::Result;

pub fn run(id: Option<i32>) -> Result<()> {
    match id {
        Some(pid) => {
            // 1. Clean specific project
            println!("Cleaning cache for project ID {}", pid);

            let project =
                crate::db::get_project_by_id(pid)?.expect("No such project found with that id");

            crate::utils::clean_dir(&project.cache_dir)?;
            crate::db::update_last_cleaned(pid)?;
            println!("Cleaned!");
        }
        None => {
            // 2. Clean all caches
            todo!();
        }
    }

    Ok(())
}
