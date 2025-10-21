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
    Clean{id: Option<u32>},
    Remove{id: u32 },
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

    pub fn run(&self) {
        match &self.command {
            Commands::Init { path } => crate::commands::init::run(path.clone()),
            Commands::Clean{id} => crate::commands::clean::run(id.clone()),
            Commands::Remove {id}=> crate::commands::remove::run(id),
            _ => {},
        }
    }
}
