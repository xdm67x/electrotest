use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "electrotest")]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

impl Cli {
    pub fn command(&self) -> Option<&Commands> {
        self.command.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum Commands {
    Init,
    Doctor,
    List,
    Test,
}

pub fn parse_args<I, T>(args: I) -> Cli
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    Cli::parse_from(args)
}

pub async fn run() -> Result<(), crate::Error> {
    let _ = parse_args(std::env::args_os());
    Ok(())
}
