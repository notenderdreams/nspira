use crate::db::{project_exists, remove_project};
use crate::utils::logger::{error, success};

pub fn run(id: i32) -> anyhow::Result<()> {
    if !project_exists(id)? {
        error(&format!("No project found with ID {}", id));
        return anyhow::Ok(());
    }

    remove_project(id)?;

    success(&format!("Removed project with ID {}", id));

    anyhow::Ok(())
}
