use camino::Utf8Path;
use tokio::process::Command;

pub async fn run() -> Result<(), crate::Error> {
    ensure_node_available().await?;
    ensure_supported_node_version().await?;
    let runtime_dir = crate::project::bootstrap::ensure_worker_runtime().await?;
    ensure_worker_dependencies(&runtime_dir).await?;
    let config = crate::config::load_default().await?;
    crate::config::validate_paths(&config)?;
    crate::config::validate_startup(&config)?;
    println!("doctor: ok");
    Ok(())
}

async fn ensure_node_available() -> Result<(), crate::Error> {
    let status = Command::new("node").arg("--version").status().await;

    match status {
        Ok(status) if status.success() => Ok(()),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Node.js is required but was not found on PATH",
        )
        .into()),
    }
}

async fn ensure_supported_node_version() -> Result<(), crate::Error> {
    let output = Command::new("node").arg("--version").output().await?;
    let version = String::from_utf8_lossy(&output.stdout);

    let major = version
        .trim()
        .trim_start_matches('v')
        .split('.')
        .next()
        .and_then(|value| value.parse::<u64>().ok());

    match major {
        Some(18 | 20 | 22) => Ok(()),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Node.js version {version:?} is not supported"),
        )
        .into()),
    }
}

async fn ensure_worker_dependencies(runtime_dir: &Utf8Path) -> Result<(), crate::Error> {
    let package_json = runtime_dir.join("package.json");
    if package_json.exists() {
        let playwright = runtime_dir.join("node_modules/playwright");
        if !playwright.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "missing Playwright worker dependency: {}",
                    playwright.as_str()
                ),
            )
            .into());
        }
    }

    Ok(())
}
