mod init;
mod run;

use clap::Subcommand;

use crate::cmd::{init::InitCommand, run::RunCommand};

pub trait Command {
    fn execute(self) -> anyhow::Result<()>;
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    Run,
}

pub fn execute(cmd: Commands) -> anyhow::Result<()> {
    match cmd {
        Commands::Init => InitCommand.execute(),
        Commands::Run => RunCommand.execute(),
    }
}
