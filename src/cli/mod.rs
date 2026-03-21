use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "electrotest")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    Doctor,
    List,
    Test,
}

pub async fn run() -> Result<(), crate::Error> {
    let _ = Cli::parse();
    Ok(())
}
