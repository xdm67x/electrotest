pub mod args;
pub mod context;
pub mod feature;
pub mod launcher;
pub mod parser;
pub mod runner;
pub mod steps;

use crate::cdp::CdpClient;
use crate::cli::context::Context;
use crate::cli::launcher::AppLauncher;
use crate::cli::parser::parse_feature_file;
use crate::cli::runner::FeatureRunner;
use anyhow::{Context as _, Result};
use args::CliArgs;
use clap::Parser;

/// Run the CLI application
pub async fn run() -> Result<()> {
    let args = CliArgs::parse();
    
    // Validate arguments
    args.validate()?;

    // Create context with output directory
    let output_dir = if args.output_dir.is_absolute() {
        args.output_dir.clone()
    } else {
        std::env::current_dir()?.join(&args.output_dir)
    };

    let (cdp_port, launcher, _pid) = if args.is_launch_mode() {
        // Mode launch: start Electron, run tests, then kill it
        launch_mode(&args).await?
    } else {
        // Mode connect: connect to existing Electron process
        connect_mode(&args).await?
    };

    // Connect to CDP
    let mut cdp_client = CdpClient::new(cdp_port);
    cdp_client.connect().await?;
    println!("✅ Connected to Electron via CDP");

    let mut ctx = Context::new(cdp_client, output_dir.clone());

    // Parse feature file
    println!("\n📄 Loading feature file: {}", args.features.display());
    let feature = parse_feature_file(&args.features)?;

    // Run the feature
    let runner = FeatureRunner::new();
    let result = runner.run_feature(&feature, &mut ctx).await?;

    // Disconnect
    ctx.cdp_client.disconnect().await?;

    // In launch mode, kill the Electron process
    if let Some(mut la) = launcher {
        la.kill()?;
    }

    // Exit with appropriate code
    let all_passed = result.scenarios.iter().all(|r| r.passed);
    if all_passed {
        println!("\n🎉 All scenarios passed!");
        Ok(())
    } else {
        println!("\n💥 Some scenarios failed");
        std::process::exit(1);
    }
}

/// Launch mode: start Electron with --remote-debugging-port, wait for CDP
async fn launch_mode(args: &CliArgs) -> Result<(u16, Option<AppLauncher>, u32)> {
    let electron_path = args.resolve_electron_path()?;
    let app_path = args.app_path.as_ref().unwrap();
    
    // Find an available port (auto-increment from base port)
    let port = AppLauncher::find_available_port(args.port).await?;
    println!("🔗 Using CDP port: {}", port);
    
    // Launch the Electron app
    let launcher = AppLauncher::launch(
        &electron_path,
        app_path,
        port,
        &args.app_args,
    )?;
    
    let pid = launcher.pid();
    println!("🔍 Launched Electron process {}...", pid);

    // Wait for CDP to become available
    launcher.wait_for_cdp().await?;

    Ok((port, Some(launcher), pid))
}

/// Connect mode: connect to existing Electron process by PID
async fn connect_mode(args: &CliArgs) -> Result<(u16, Option<AppLauncher>, u32)> {
    let pid = args.pid.unwrap();
    println!("🔍 Connecting to Electron process {}...", pid);

    // Get CDP port from process
    let cdp_port = find_cdp_port(pid).await?;
    println!("🔗 Found CDP port: {}", cdp_port);

    Ok((cdp_port, None, pid))
}

/// Find the CDP port for a given Electron PID
async fn find_cdp_port(pid: u32) -> Result<u16> {
    use sysinfo::{ProcessesToUpdate, System};

    let mut system = System::new_all();
    system.refresh_processes(ProcessesToUpdate::All, true);

    let process = system
        .process(sysinfo::Pid::from_u32(pid))
        .ok_or_else(|| anyhow::anyhow!("Process with PID {} not found", pid))?;

    // Extract CDP port from command line arguments
    let cmd: Vec<String> = process
        .cmd()
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect();
    let cmd_str = cmd.join(" ");

    // Look for --remote-debugging-port=XXXX
    if let Some(pos) = cmd_str.find("--remote-debugging-port=") {
        let start = pos + "--remote-debugging-port=".len();
        let end = cmd_str[start..]
            .find(' ')
            .map(|i| start + i)
            .unwrap_or(cmd_str.len());
        let port_str = &cmd_str[start..end];
        return port_str
            .parse::<u16>()
            .with_context(|| format!("Invalid CDP port: {}", port_str));
    }

    // Also check individual command parts
    for part in &cmd {
        if let Some(port_str) = part.strip_prefix("--remote-debugging-port=") {
            return port_str
                .parse::<u16>()
                .with_context(|| format!("Invalid CDP port: {}", port_str));
        }
    }

    Err(anyhow::anyhow!(
        "Could not find --remote-debugging-port in process {}. \
         Make sure Electron was started with --remote-debugging-port flag",
        pid
    ))
}
