pub mod cli;
mod commands;
mod db;
pub mod utils;

const DEFAULT_PATTERNS: &str = include_str!("../patterns.json");
