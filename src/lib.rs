pub mod cli;
pub mod config;
pub mod core;
pub mod db;
mod commands;
pub mod utils;

pub const DEFAULT_PATTERNS: &str = include_str!("../patterns.json");
