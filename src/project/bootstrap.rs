use camino::Utf8PathBuf;

pub async fn ensure_worker_runtime() -> Result<Utf8PathBuf, crate::Error> {
    let cache_dir = if let Some(cache_dir) = std::env::var_os("ELECTROTEST_CACHE_DIR") {
        std::path::PathBuf::from(cache_dir).join("worker/v1")
    } else {
        directories::ProjectDirs::from("dev", "memo", "electrotest")
            .unwrap()
            .cache_dir()
            .join("worker/v1")
    };

    tokio::fs::create_dir_all(&cache_dir).await?;
    Ok(Utf8PathBuf::from_path_buf(cache_dir).unwrap())
}
