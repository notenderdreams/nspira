pub fn run() -> anyhow::Result<()> {
    let projects = crate::db::get_all_projects()?;
    crate::utils::print_projects(projects);
    anyhow::Ok(())
}
