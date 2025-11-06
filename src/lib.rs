pub mod cli;
pub mod config;
pub mod core;
pub mod db;
pub mod ui;
mod commands;
pub mod utils;

pub const DEFAULT_PATTERNS: &str = include_str!("../patterns.json");
