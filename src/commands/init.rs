use std::path::PathBuf;

pub fn run(path: Option<PathBuf>) {
    //  1. Determine target path
    let project_path = path.unwrap_or_else(|| std::env::current_dir().unwrap());

    // 2. Insert into DB
    // crate::db::add_project(&project_path);

    //  3. Confirm
    println!("Initialized project at {}", project_path.display());
}
