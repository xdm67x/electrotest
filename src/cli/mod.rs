use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

mod commands {
    pub mod doctor;
    pub mod init;
    pub mod list;
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

#[derive(Args, Default)]
pub struct DoctorArgs {}

#[derive(Args, Default)]
pub struct ListArgs {
    #[arg(long, value_name = "PATH")]
    pub features: Vec<PathBuf>,
}

#[derive(Args, Default)]
pub struct TestArgs {}

#[derive(Subcommand)]
pub enum Commands {
    Init(InitArgs),
    Doctor(DoctorArgs),
    List(ListArgs),
    Test(TestArgs),
}

pub async fn run() -> Result<(), crate::Error> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Test(TestArgs::default())) {
        Commands::Init(args) => commands::init::run(&args.path).await,
        Commands::Doctor(_) => commands::doctor::run().await,
        Commands::List(args) => commands::list::run(args).await,
        Commands::Test(args) => commands::test::run(args).await,
    }
}
