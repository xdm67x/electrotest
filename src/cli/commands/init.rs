use std::path::Path;

use crate::project::templates;

pub async fn run(path: &Path) -> Result<(), crate::Error> {
    tokio::fs::create_dir_all(path.join("features")).await?;
    tokio::fs::create_dir_all(path.join("steps")).await?;
    tokio::fs::write(path.join("electrotest.toml"), templates::DEFAULT_CONFIG).await?;
    tokio::fs::write(path.join("tsconfig.json"), templates::DEFAULT_TSCONFIG).await?;
    Ok(())
}
