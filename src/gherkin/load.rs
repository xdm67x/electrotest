use std::path::Path;

use super::{compile_str, CompileError, CompiledScenario};

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Compile(#[from] CompileError),
}

pub fn load_scenarios<I, P>(feature_paths: I) -> Result<Vec<CompiledScenario>, LoadError>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut scenarios = Vec::new();

    for feature_path in feature_paths {
        let feature_path = feature_path.as_ref();
        if feature_path.is_dir() {
            for entry in std::fs::read_dir(feature_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("feature") {
                    scenarios.extend(load_scenarios_from_file(&path)?);
                }
            }
        } else {
            scenarios.extend(load_scenarios_from_file(feature_path)?);
        }
    }

    Ok(scenarios)
}

fn load_scenarios_from_file(path: &Path) -> Result<Vec<CompiledScenario>, LoadError> {
    let raw = std::fs::read_to_string(path)?;
    Ok(compile_str(&raw)?.scenarios)
}
