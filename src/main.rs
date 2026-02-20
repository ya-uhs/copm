use clap::Parser;

use copm::cli::args::Cli;
use copm::commands;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(e) = commands::dispatch(cli.command).await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
