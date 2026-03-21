mod types;
mod validate;

use std::path::Path;

use camino::Utf8Path;

pub use self::types::{AppConfig, AppMode, Config, PathsConfig};
pub use self::validate::ConfigError;

const DEFAULT_CONFIG_FILE: &str = "electrotest.toml";

pub fn from_str(raw: &str) -> Result<Config, ConfigError> {
    let config: Config = toml::from_str(raw).map_err(ConfigError::from)?;
    validate::validate(&config)?;
    Ok(config)
}

pub fn from_path(path: &Path) -> Result<Config, ConfigError> {
    let raw = std::fs::read_to_string(path)?;
    let mut config = from_str(&raw)?;

    let base = path.parent().ok_or(ConfigError::MissingConfigDirectory)?;
    let base = Utf8Path::from_path(base).ok_or(ConfigError::NonUtf8ConfigPath)?;

    resolve_paths(&mut config, base);
    Ok(config)
}

pub async fn load_default() -> Result<Config, ConfigError> {
    let path = Path::new(DEFAULT_CONFIG_FILE);
    if !path.exists() {
        return Err(ConfigError::MissingConfigFile);
    }

    from_path(path)
}

pub fn validate_paths(config: &Config) -> Result<(), ConfigError> {
    validate::validate_paths(config)
}

pub fn validate_startup(config: &Config) -> Result<(), ConfigError> {
    validate::validate_startup(config)
}

fn resolve_paths(config: &mut Config, base: &Utf8Path) {
    if let Some(endpoint_file) = config.app.endpoint_file.take() {
        config.app.endpoint_file = Some(resolve_path(base, endpoint_file));
    }

    config.paths.features = config
        .paths
        .features
        .drain(..)
        .map(|path| resolve_path(base, path))
        .collect();
    config.paths.steps = config
        .paths
        .steps
        .drain(..)
        .map(|path| resolve_path(base, path))
        .collect();
    config.paths.artifacts = resolve_path(base, std::mem::take(&mut config.paths.artifacts));
}

fn resolve_path(base: &Utf8Path, path: camino::Utf8PathBuf) -> camino::Utf8PathBuf {
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}
