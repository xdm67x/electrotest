use clap::Parser;
use std::path::PathBuf;

/// CLI automation for Electron applications using CDP and Gherkin scenarios
#[derive(Parser, Debug)]
#[command(name = "electrotest")]
#[command(about = "Run Gherkin scenarios against Electron applications")]
pub struct CliArgs {
    /// PID of the Electron process to connect to
    #[arg(short, long)]
    pub pid: u32,

    /// Path to the .feature file
    #[arg(short, long)]
    pub features: PathBuf,

    /// Output directory for screenshots
    #[arg(short, long, default_value = "./output")]
    pub output_dir: PathBuf,
}
