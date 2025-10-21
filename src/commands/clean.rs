// commands/clean.rs
pub fn run(id: Option<u32>) {
    match id {
        Some(pid) => {
            // 1. Clean specific project
            println!("Cleaning cache for project ID {}", pid);
            // let project = crate::db::get_project_by_id(pid);
            // crate::utils::clean_dir(&project.cache_path);
        }
        None => {
            //  2. Clean all caches
            println!("Cleaning all project caches...");
            // for project in crate::db::get_all_projects() {
            //     crate::utils::clean_dir(&project.cache_path);
            // }
        }
    }
}
