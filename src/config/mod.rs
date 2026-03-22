use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const FILENAME: &str = "electrotest.toml";

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
    Bun,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub package_manager: PackageManager,
}

fn config_filepath() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(FILENAME))
}

pub fn create() -> anyhow::Result<()> {
    let default_config = Config {
        package_manager: PackageManager::Npm,
    };
    let path = config_filepath()?;
    if !path.exists() {
        let mut file = File::create(path)?;
        let toml = toml::to_string_pretty(&default_config)?;
        file.write_all(toml.as_bytes())?;
    }
    Ok(())
}

pub fn parse() -> anyhow::Result<Config> {
    let path = config_filepath()?;
    let input = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&input)?;
    Ok(config)
}
