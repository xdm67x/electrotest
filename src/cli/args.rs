use clap::Parser;
use std::path::PathBuf;

use super::launcher;

/// CLI automation for Electron applications using CDP and Gherkin scenarios
#[derive(Parser, Debug)]
#[command(name = "electrotest")]
#[command(about = "Run Gherkin scenarios against Electron applications")]
pub struct CliArgs {
    /// PID of an already running Electron process to connect to
    #[arg(short, long, conflicts_with_all = ["electron_path", "app_path"])]
    pub pid: Option<u32>,

    /// Path to the Electron executable (optional - auto-detected from app_path if not provided)
    #[arg(long, conflicts_with = "pid")]
    pub electron_path: Option<PathBuf>,

    /// Path to the Electron application directory or main file
    #[arg(long, conflicts_with = "pid")]
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
    /// Resolves the Electron executable path.
    /// If electron_path is explicitly provided, returns it.
    /// Otherwise, attempts to auto-detect from app_path's node_modules/.bin/
    pub fn resolve_electron_path(&self) -> anyhow::Result<PathBuf> {
        if let Some(ref p) = self.electron_path {
            return Ok(p.clone());
        }
        if let Some(ref app_path) = self.app_path
            && let Some(detected) = launcher::detect_electron_path(app_path)
        {
            return Ok(detected);
        }
        anyhow::bail!(
            "Could not auto-detect Electron executable. \
             Provide --electron-path explicitly, or ensure Electron is installed \
             in node_modules at the app path: {}",
            self.app_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        );
    }

    /// Validate that either pid or (electron_path or app_path) is provided
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.pid.is_some() {
            // Mode connect: pid is provided, electron_path and app_path should be None
            if self.electron_path.is_some() || self.app_path.is_some() {
                anyhow::bail!("Cannot specify both --pid and (--electron-path/--app-path)");
            }
            Ok(())
        } else if self.electron_path.is_some() || self.app_path.is_some() {
            // Mode launch: at least one of electron_path or app_path must be provided
            // (electron_path can be auto-detected from app_path)
            Ok(())
        } else {
            anyhow::bail!(
                "Must specify either --pid or --app-path (with optional --electron-path). \
                 Use --pid for connecting to a running Electron process, \
                 or --app-path (with optional --electron-path) to launch and test an application."
            );
        }
    }

    /// Returns true if in launch mode (electron_path or app_path provided)
    pub fn is_launch_mode(&self) -> bool {
        self.electron_path.is_some() || self.app_path.is_some()
    }
}
