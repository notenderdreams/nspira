use clap::Parser;
use nspira::cli::Cli;
use nspira::utils::logger;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.run() {
        logger::error(&e.to_string());
        std::process::exit(1);
    }
}
