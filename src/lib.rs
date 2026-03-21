pub mod cli;
pub mod config;
pub mod gherkin;
pub mod project {
    pub mod bootstrap;
    pub mod templates;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),
}
