use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

mod commands {
    pub mod init;
}

#[derive(Parser)]
#[command(name = "electrotest")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Args)]
pub struct InitArgs {
    #[arg(long, default_value = ".")]
    path: PathBuf,
}

#[derive(Subcommand)]
pub enum Commands {
    Init(InitArgs),
    Doctor,
    List,
    Test,
}

pub async fn run() -> Result<(), crate::Error> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init(args)) => commands::init::run(&args.path).await,
        _ => Ok(()),
    }
}
