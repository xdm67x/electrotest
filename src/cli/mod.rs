use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

mod commands {
    pub mod doctor;
    pub mod init;
    pub mod test;
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
        Some(Commands::Doctor) => commands::doctor::run().await,
        Some(Commands::Test) => commands::test::run().await,
        _ => Ok(()),
    }
}
