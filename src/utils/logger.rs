use colored::*;
use std::io::{self, Write};

/// Prompt the user for input
pub fn ask_input(prompt: &str) -> String {
    let label = ">>".cyan().bold();
    print!("{} {}: ", label, prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

/// Print an informational message
pub fn info(msg: &str) {
    let label = "INFO:".black().on_bright_blue().bold();
    println!("{} {}", label, msg);
}

/// Print a success message
pub fn success(msg: &str) {
    let label = "SUCCESS\t:".black().on_bright_green().bold();
    println!("{} {}", label, msg);
}

/// Print a warning message
pub fn warn(msg: &str) {
    let label = "WARNING\t:".black().on_bright_yellow().bold();
    println!("{} {}", label, msg);
}

/// Print an error message
pub fn error(msg: &str) {
    let label = "ERROR\t:".black().on_bright_red().bold();
    eprintln!("{} {}", label, msg);
}


pub fn task(msg: &str) {
    let label = "TASK:".black().on_bright_magenta().bold();
    println!("{} {}", label, msg);
}