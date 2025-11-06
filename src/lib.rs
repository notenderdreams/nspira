pub mod cli;
mod commands;
pub mod config;
pub mod core;
pub mod db;
pub mod ui;
pub mod utils;

pub const DEFAULT_PATTERNS: &str = include_str!("../patterns.json");
