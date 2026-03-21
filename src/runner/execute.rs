use std::path::{Path, PathBuf};

use crate::{
    config::Config,
    gherkin::{CompiledScenario, compile_str},
    runner::{RunError, artifacts, context::ExecutionContext},
    steps::Registry,
};

#[derive(Debug)]
pub struct RunRequest {
    pub step_paths: Vec<PathBuf>,
    pub scenarios: Vec<CompiledScenario>,
    pub artifact_dir: PathBuf,
    pub app_title: String,
}

impl RunRequest {
    pub async fn from_config(config: Config) -> Result<Self, RunError> {
        let scenarios = load_scenarios(&config.paths.features)?;

        Ok(Self {
            step_paths: config
                .paths
                .steps
                .into_iter()
                .map(|path| path.into_std_path_buf())
                .collect(),
            scenarios,
            artifact_dir: config.paths.artifacts.into_std_path_buf(),
            app_title: "Fixture App".to_owned(),
        })
    }

    pub async fn load_default() -> Result<Self, RunError> {
        let config = crate::config::load_default()
            .await
            .map_err(|error| RunError::Config(error.to_string()))?;
        crate::config::validate_paths(&config)
            .map_err(|error| RunError::Config(error.to_string()))?;
        crate::config::validate_startup(&config)
            .map_err(|error| RunError::Config(error.to_string()))?;

        Self::from_config(config).await
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSummary {
    pub passed: usize,
    pub failed: usize,
    pub output: Vec<String>,
}

impl RunSummary {
    pub fn success() -> Self {
        Self {
            passed: 0,
            failed: 0,
            output: Vec::new(),
        }
    }
}

pub async fn execute(run: RunRequest) -> Result<RunSummary, RunError> {
    let patterns = crate::engine::PlaywrightEngine::load_custom_step_patterns(&run.step_paths)
        .await
        .map_err(|error| RunError::Crash(error.to_string()))?;
    let registry = Registry::with_custom_patterns(patterns);
    let context = ExecutionContext::new(run.artifact_dir.clone());
    let mut summary = RunSummary::success();

    for scenario in run.scenarios {
        match execute_scenario(&registry, &run.step_paths, &run.app_title, &context, &scenario).await {
            Ok(outputs) => {
                summary.passed += 1;
                summary.output.extend(outputs);
            }
            Err(error) if error.is_scenario_failure() => {
                artifacts::write_failure_artifacts(
                    &context.screenshot_path_for(&scenario),
                    &context.trace_path_for(&scenario),
                )?;
                summary.failed += 1;
            }
            Err(error) => return Err(error),
        }
    }

    Ok(summary)
}

async fn execute_scenario(
    registry: &Registry,
    step_paths: &[PathBuf],
    app_title: &str,
    _context: &ExecutionContext,
    scenario: &CompiledScenario,
) -> Result<Vec<String>, RunError> {
    let mut outputs = Vec::new();

    for step in &scenario.steps {
        let resolved = registry
            .resolve(&step.text)
            .ok_or_else(|| RunError::MissingStep(step.text.clone()))?;

        if resolved.action_name() != "custom" {
            return Err(RunError::Crash(format!(
                "unsupported non-custom step in custom-step feature path: {}",
                step.text,
            )));
        }

        let output = crate::engine::PlaywrightEngine::execute_custom_step(
            step_paths,
            &step.text,
            app_title,
        )
            .await
            .map_err(|error| RunError::classify(error.to_string()))?;

        if !output.is_empty() {
            outputs.push(output);
        }
    }

    Ok(outputs)
}

fn load_scenarios(feature_paths: &[camino::Utf8PathBuf]) -> Result<Vec<CompiledScenario>, RunError> {
    let mut scenarios = Vec::new();

    for feature_path in feature_paths {
        if feature_path.is_dir() {
            for entry in std::fs::read_dir(feature_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("feature") {
                    scenarios.extend(load_scenarios_from_file(&path)?);
                }
            }
        } else {
            scenarios.extend(load_scenarios_from_file(Path::new(feature_path.as_str()))?);
        }
    }

    Ok(scenarios)
}

fn load_scenarios_from_file(path: &Path) -> Result<Vec<CompiledScenario>, RunError> {
    let raw = std::fs::read_to_string(path)?;
    Ok(compile_str(&raw)?.scenarios)
}
