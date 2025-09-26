use clap::Parser;
use std::process;

// Declaration of all local modules
mod app;
mod cli;
mod error;
mod handlers;
mod models;
mod storage;
mod ui;

use cli::{Cli, Commands};
use error::Result;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List(args) => handlers::handle_list(args),
        Commands::Add(args) => handlers::handle_add(args),
        Commands::Delete(args) => handlers::handle_delete(args),
        Commands::Search(args) => handlers::handle_search(args),
    }
}
