use crate::{db, utils};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use utils::logger::*;

#[derive(Parser)]
#[command(
    name = "nspira",
    version,
    before_help = r#"
    ┌────────────────────────────┐
    │     🌿  n s p i r a  🌿      │
    └────────────────────────────┘
Lightweight cache manager for developers
    "#,
    after_help = "Examples:\n  \
                  nspira init                           Auto-detect project in current directory\n  \
                  nspira add myapp target node_modules  Add project with multiple cache dirs\n  \
                  nspira list                           Show all tracked caches\n  \
                  nspira clean                          Clean all caches safely\n  \
                  nspira stats                          View storage statistics"

)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new project in the current directory
    Init {
        /// Optional cache directory path
        path: Option<PathBuf>
    },
    /// Add a new project with cache directories
    Add {
        /// Project name
        name: String,
        /// One or more cache directory paths
        #[arg(required = true, num_args = 1..)]
        cache_paths: Vec<PathBuf>
    },
    /// Clean cache directories
    Clean {
        /// Optional project ID (cleans all if not provided)
        id: Option<i32>
    },
    /// Remove a project from tracking
    Remove {
        /// Project ID to remove
        id: i32
    },
    /// List all tracked projects (interactive TUI)
    List,
    /// Show cache statistics
    Stats,
    /// Check database and project health
    Doctor,
    /// Delete the entire database
    Flush,
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }

    pub fn run(&self) -> anyhow::Result<()> {
        if let Err(e) = db::init_db() {
            error(&format!("Failed to initialize database: {}", e));
            std::process::exit(1);
        }

        match &self.command {
            Commands::Init { path } => crate::commands::init::run(path.clone())?,
            Commands::Add { name, cache_paths } => {
                crate::commands::add::run(name, cache_paths.clone())?
            }
            Commands::Clean { id } => crate::commands::clean::run(*id)?,
            Commands::Remove { id } => crate::commands::remove::run(*id)?,
            Commands::List => crate::commands::list::run()?,
            Commands::Stats => crate::commands::stats::run()?,
            Commands::Doctor => crate::commands::doctor::run()?,
            Commands::Flush => crate::commands::flush::run()?,
        }

        Ok(())
    }
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}