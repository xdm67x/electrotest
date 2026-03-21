use camino::Utf8PathBuf;
use std::path::{Path, PathBuf};
use tokio::process::Command;

const PACKAGE_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/runtime/worker/package.json"));
const TSCONFIG_JSON: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/runtime/worker/tsconfig.json"));
const INDEX_TS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/runtime/worker/src/index.ts"));
const PROTOCOL_TS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/runtime/worker/src/protocol.ts"));
const ENGINE_TS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/runtime/worker/src/engine.ts"));
const BUILD_MJS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/runtime/worker/scripts/build.mjs"));

pub async fn ensure_worker_runtime() -> Result<Utf8PathBuf, crate::Error> {
    let cache_dir = worker_cache_dir();
    tokio::fs::create_dir_all(&cache_dir).await?;
    utf8_path(cache_dir)
}

pub async fn materialize_runtime(cache_root: &Path) -> Result<Utf8PathBuf, crate::Error> {
    copy_embedded_runtime(cache_root).await?;
    run_npm_install(cache_root).await?;
    run_worker_build(cache_root).await?;
    utf8_path(cache_root.join("dist"))
}

fn worker_cache_dir() -> PathBuf {
    if let Some(cache_dir) = std::env::var_os("ELECTROTEST_CACHE_DIR") {
        PathBuf::from(cache_dir).join("worker/v1")
    } else {
        directories::ProjectDirs::from("dev", "memo", "electrotest")
            .unwrap()
            .cache_dir()
            .join("worker/v1")
    }
}

async fn copy_embedded_runtime(cache_root: &Path) -> Result<(), crate::Error> {
    tokio::fs::create_dir_all(cache_root.join("src")).await?;
    tokio::fs::create_dir_all(cache_root.join("scripts")).await?;

    tokio::fs::write(cache_root.join("package.json"), PACKAGE_JSON).await?;
    tokio::fs::write(cache_root.join("tsconfig.json"), TSCONFIG_JSON).await?;
    tokio::fs::write(cache_root.join("src/index.ts"), INDEX_TS).await?;
    tokio::fs::write(cache_root.join("src/protocol.ts"), PROTOCOL_TS).await?;
    tokio::fs::write(cache_root.join("src/engine.ts"), ENGINE_TS).await?;
    tokio::fs::write(cache_root.join("scripts/build.mjs"), BUILD_MJS).await?;

    Ok(())
}

async fn run_npm_install(cache_root: &Path) -> Result<(), crate::Error> {
    run_command(cache_root, "npm", &["install"]).await
}

async fn run_worker_build(cache_root: &Path) -> Result<(), crate::Error> {
    run_command(cache_root, "npm", &["run", "build"]).await
}

async fn run_command(cache_root: &Path, program: &str, args: &[&str]) -> Result<(), crate::Error> {
    let status = Command::new(program)
        .args(args)
        .current_dir(cache_root)
        .status()
        .await?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "{program} {} failed with status {status}",
            args.join(" ")
        ))
        .into())
    }
}

fn utf8_path(path: PathBuf) -> Result<Utf8PathBuf, crate::Error> {
    Utf8PathBuf::from_path_buf(path).map_err(|path| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("path contains invalid UTF-8: {}", path.display()),
        )
        .into()
    })
}
