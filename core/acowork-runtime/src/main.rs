//! acowork-runtime CLI entry point
use clap::Parser;
use acowork_runtime::cli::Cli;

fn main() -> acowork_runtime::error::Result<()> {
    let cli = Cli::parse();
    cli.run()
}
