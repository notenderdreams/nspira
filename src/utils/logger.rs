use colored::*;
use std::io::{self, Write};

/// Prompt the user for input
pub fn ask_input(prompt: &str) -> String {
    print!("{PAD}{} {}: ", ">".cyan().bold(), prompt);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    input.trim().to_string()
}

const PAD: &str = "  ";

/// Print an informational message:  `  info: msg`
pub fn info(msg: &str) {
    println!("{PAD}{} {}", "info:".blue(), msg);
}

/// Print a success message:  `  ok: msg`
pub fn success(msg: &str) {
    println!("{PAD}{} {}", "ok:".green(), msg.bold());
}

/// Print a warning to stderr:  `  warn: msg`
pub fn warn(msg: &str) {
    eprintln!("{PAD}{} {}", "warn:".yellow(), msg);
}

/// Print an error to stderr:  `  error: msg`
pub fn error(msg: &str) {
    eprintln!("{PAD}{} {}", "error:".red(), msg);
}

/// Print a task/progress line:  `  task: msg`
pub fn task(msg: &str) {
    println!("{PAD}{} {}", "task:".magenta(), msg);
}
