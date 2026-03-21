use std::path::PathBuf;

use crate::gherkin::CompiledScenario;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    artifact_dir: PathBuf,
}

impl ExecutionContext {
    pub fn new(artifact_dir: PathBuf) -> Self {
        Self { artifact_dir }
    }

    pub fn screenshot_path_for(&self, _scenario: &CompiledScenario) -> PathBuf {
        self.artifact_dir.join("failure.png")
    }

    pub fn trace_path_for(&self, _scenario: &CompiledScenario) -> PathBuf {
        self.artifact_dir.join("trace.zip")
    }
}
