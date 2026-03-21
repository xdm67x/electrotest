#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("config error: {0}")]
    Config(String),
    #[error("missing step: {0}")]
    MissingStep(String),
    #[error("element not found: {0}")]
    ElementNotFound(String),
    #[error("timeout: {0}")]
    Timeout(String),
    #[error("crash: {0}")]
    Crash(String),
    #[error("assertion failed: {0}")]
    Assertion(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Gherkin(#[from] crate::gherkin::CompileError),
}

impl RunError {
    pub fn classify(message: impl Into<String>) -> Self {
        let message = message.into();
        let lower = message.to_ascii_lowercase();

        if lower.contains("no step matched") {
            Self::MissingStep(message)
        } else if lower.contains("element not found") {
            Self::ElementNotFound(message)
        } else if lower.contains("timeout") {
            Self::Timeout(message)
        } else if lower.contains("assert") || lower.contains("expected") {
            Self::Assertion(message)
        } else {
            Self::Crash(message)
        }
    }

    pub fn is_scenario_failure(&self) -> bool {
        matches!(
            self,
            Self::Assertion(_)
                | Self::ElementNotFound(_)
                | Self::Timeout(_)
                | Self::MissingStep(_)
                | Self::Crash(_)
        )
    }
}
