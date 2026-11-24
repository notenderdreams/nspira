use crate::db;
use crate::utils::logger::{info, success, warn};
use anyhow::Result;
use std::path::PathBuf;

fn absolute(path: &PathBuf) -> Result<String> {
    Ok(crate::utils::absolute(path).to_string_lossy().to_string())
}

pub fn list() -> Result<()> {
    let paths = db::get_ignored(&db::connect()?)?;
    if paths.is_empty() {
        info("No ignored paths");
    }
    for p in paths {
        println!("  {}", p);
    }
    Ok(())
}

pub fn add(path: &PathBuf) -> Result<()> {
    let path = absolute(path)?;
    if db::add_ignored(&db::connect()?, &path)? {
        success(&format!("Ignoring {} in future scans", path));
    } else {
        info(&format!("{} is already ignored", path));
    }
    Ok(())
}

pub fn remove(path: &PathBuf) -> Result<()> {
    let conn = db::connect()?;
    // Accept the stored string as-is too, in case the path no longer exists.
    let raw = path.to_string_lossy().to_string();
    if db::remove_ignored(&conn, &raw)? || db::remove_ignored(&conn, &absolute(path)?)? {
        success(&format!("{} will be scanned again", raw));
    } else {
        warn(&format!("{} was not ignored", raw));
    }
    Ok(())
}
