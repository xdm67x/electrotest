use super::types::{AppMode, Config};

use camino::Utf8PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing launch command")]
    MissingLaunchCommand,
    #[error("missing attach endpoint")]
    MissingAttachEndpoint,
    #[error("conflicting attach endpoint sources")]
    ConflictingAttachEndpointSources,
    #[error("config parse error: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("config path must include a parent directory")]
    MissingConfigDirectory,
    #[error("config path is not valid UTF-8")]
    NonUtf8ConfigPath,
    #[error("missing config file: electrotest.toml")]
    MissingConfigFile,
    #[error("missing path: {0}")]
    MissingPath(Utf8PathBuf),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn validate(config: &Config) -> Result<(), ConfigError> {
    validate_startup(config)
}

pub fn validate_startup(config: &Config) -> Result<(), ConfigError> {
    match config.app.mode {
        AppMode::Launch if config.app.command.is_none() => Err(ConfigError::MissingLaunchCommand),
        AppMode::Attach if config.app.endpoint.is_none() && config.app.endpoint_file.is_none() => {
            Err(ConfigError::MissingAttachEndpoint)
        }
        AppMode::Attach if config.app.endpoint.is_some() && config.app.endpoint_file.is_some() => {
            Err(ConfigError::ConflictingAttachEndpointSources)
        }
        _ => Ok(()),
    }
}

pub fn validate_paths(config: &Config) -> Result<(), ConfigError> {
    for path in config
        .paths
        .features
        .iter()
        .chain(config.paths.steps.iter())
    {
        if !path.exists() {
            return Err(ConfigError::MissingPath(path.clone()));
        }
    }

    Ok(())
}
