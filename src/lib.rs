pub mod cli;
mod commands;
pub mod core;
pub mod db;
pub mod utils;

pub const DEFAULT_PATTERNS: &str = include_str!("../patterns.json");
