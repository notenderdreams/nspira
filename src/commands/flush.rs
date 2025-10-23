use colored::Colorize;
use crate::db::flush_db;
use crate::utils::logger::{ask_input, info};

pub fn run() ->anyhow::Result<()>{
    let confirmation = ask_input(
        &format!("{} Continue? (y/n)"," This will permanently delete all tracked projects.".red())
    );
    if confirmation.to_lowercase() != "y" {
        info("Flush Cancelled!");
        return Ok(());
    }
    flush_db()
}