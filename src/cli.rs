use crate::db;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nspira", about = "Manage projects and caches easily")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init { path: Option<PathBuf> },
    Add { name: String, cache_path: PathBuf },
    Clean { id: Option<i32> },
    Remove { id: i32 },
    List,
    Stats,
    Doctor,
    // Search{keyword: Option<String>},
    // Export{path: PathBuf},
    // Import{path: PathBuf},
    //
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }

    pub fn run(&self) -> anyhow::Result<()> {
        if let Err(e) = db::init_db() {
            eprintln!("ERROR: Failed to initialize database: {}", e);
            std::process::exit(1);
        }

        match &self.command {
            Commands::Init { path } => crate::commands::init::run(path.clone())?,
            Commands::Add { name , cache_path } => crate::commands::add::run(name, cache_path.clone())?,
            Commands::Clean { id } => crate::commands::clean::run(id.clone())?,
            Commands::Remove { id } => crate::commands::remove::run(id.clone())?,
            Commands::List => crate::commands::list::run()?,
            Commands::Stats => crate::commands::stats::run()?,
            _ => {}
        }

        Ok(())
    }
}
