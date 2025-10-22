pub fn run(id: i32) ->anyhow::Result<()>{
    
    if !crate::db::project_exists(id)? {
        eprintln!("No project found with ID {}", id);
        return anyhow::Ok(());
    }

    crate::db::remove_project(id)?;

    println!("Removed project with ID {}", id);

    anyhow::Ok(())
}
