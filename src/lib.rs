pub mod cli;
pub mod config;
pub mod engine;
pub mod gherkin;
pub mod steps;
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
    #[error(transparent)]
    Gherkin(#[from] crate::gherkin::CompileError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
