pub mod cli;
pub mod config;
pub mod project {
    pub mod templates;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
