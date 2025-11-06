use crate::db;
use crate::utils::logger::{error, success};

pub fn run(id: i32) -> anyhow::Result<()> {
    let conn = db::connect()?;

    if !db::project_exists(&conn, id)? {
        error(&format!("No project found with ID {}", id));
        return anyhow::Ok(());
    }

    db::remove_project(&conn, id)?;

    success(&format!("Removed project with ID {}", id));

    anyhow::Ok(())
}
