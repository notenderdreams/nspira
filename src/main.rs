use clap::Parser;
use nspira::cli::Cli;

fn main() {
    let cli = Cli::parse();
    cli.run();
}