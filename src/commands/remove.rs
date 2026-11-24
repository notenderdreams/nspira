use crate::db;
use crate::utils::logger::success;
use anyhow::{Result, bail};

pub fn run(id: i32) -> Result<()> {
    if !db::remove_project(&db::connect()?, id)? {
        bail!("No project found with id {}", id);
    }
    success(&format!("Removed project {} from tracking", id));
    Ok(())
}
