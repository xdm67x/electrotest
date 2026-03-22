mod cmd;
mod config;
mod electron;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: cmd::Commands,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    cmd::execute(cli.command)?;

    Ok(())
}
