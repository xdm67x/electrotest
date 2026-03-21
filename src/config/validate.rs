use super::types::{AppMode, Config};

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
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn validate(config: &Config) -> Result<(), ConfigError> {
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
