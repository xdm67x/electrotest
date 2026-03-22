mod electron;
mod prompt;

use clap::Parser;

use crate::{electron::Electron, prompt::Prompt};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    pid: u32,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let electron = Electron::attach(cli.pid)?;
    Prompt::new(electron).run()
}
