use crate::db::flush_db;
use crate::utils::logger::{ask_input, info, success, warn};

pub fn run() -> anyhow::Result<()> {
    warn("This permanently deletes all tracked projects and ignored paths.");
    if !ask_input("Continue? (y/n)").eq_ignore_ascii_case("y") {
        info("Cancelled");
        return Ok(());
    }
    if flush_db()? {
        success("Database deleted");
    } else {
        info("No database to delete");
    }
    Ok(())
}
