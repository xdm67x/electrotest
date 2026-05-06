use clap::Parser;
use std::path::PathBuf;

/// CLI automation for Electron applications using CDP and Gherkin scenarios
#[derive(Parser, Debug)]
#[command(name = "electrotest")]
#[command(about = "Run Gherkin scenarios against Electron applications")]
pub struct CliArgs {
    /// PID of an already running Electron process to connect to
    #[arg(short, long, conflicts_with_all = ["electron_path", "app_path"])]
    pub pid: Option<u32>,

    /// Path to the Electron executable
    #[arg(long, conflicts_with = "pid", requires = "app_path")]
    pub electron_path: Option<PathBuf>,

    /// Path to the Electron application directory or main file
    #[arg(long, conflicts_with = "pid", requires = "electron_path")]
    pub app_path: Option<PathBuf>,

    /// Port for remote debugging (auto-incremented if in use)
    #[arg(long, default_value = "9222")]
    pub port: u16,

    /// Additional arguments to pass to the Electron app
    #[arg(long, default_value = "")]
    pub app_args: String,

    /// Path to the .feature file
    #[arg(short, long)]
    pub features: PathBuf,

    /// Output directory for screenshots
    #[arg(short, long, default_value = "./output")]
    pub output_dir: PathBuf,
}

impl CliArgs {
    /// Validate that either pid or (electron_path + app_path) is provided
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.pid.is_some() {
            // Mode connect: pid is provided, electron_path and app_path should be None
            if self.electron_path.is_some() || self.app_path.is_some() {
                anyhow::bail!("Cannot specify both --pid and (--electron-path/--app-path)");
            }
            Ok(())
        } else if self.electron_path.is_some() && self.app_path.is_some() {
            // Mode launch: both electron_path and app_path are provided
            Ok(())
        } else {
            anyhow::bail!(
                "Must specify either --pid or both --electron-path and --app-path. \
                 Use --pid for connecting to a running Electron process, \
                 or --electron-path and --app-path to launch and test an application."
            );
        }
    }

    /// Returns true if in launch mode (electron_path + app_path provided)
    pub fn is_launch_mode(&self) -> bool {
        self.electron_path.is_some() && self.app_path.is_some()
    }
}
