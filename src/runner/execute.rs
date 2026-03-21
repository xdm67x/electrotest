use std::path::{Path, PathBuf};

use crate::{
    config::{AppMode, Config},
    engine::{PlaywrightEngine, WorkerProcess},
    gherkin::{CompiledScenario, compile_str},
    runner::{RunError, artifacts, context::ExecutionContext},
    steps::{Registry, normalize_target},
};

#[derive(Debug)]
pub struct RunRequest {
    pub app: AppRequest,
    pub step_paths: Vec<PathBuf>,
    pub scenarios: Vec<CompiledScenario>,
    pub artifact_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub enum AppRequest {
    Launch { command: String, args: Vec<String> },
    Attach { endpoint: String },
}

impl RunRequest {
    pub async fn from_config(config: Config) -> Result<Self, RunError> {
        let scenarios = load_scenarios(&config.paths.features)?;
        let app = match config.app.mode {
            AppMode::Launch => AppRequest::Launch {
                command: config.app.command.expect("validated launch command"),
                args: config.app.args,
            },
            AppMode::Attach => AppRequest::Attach {
                endpoint: resolve_attach_endpoint(&config).await?,
            },
        };

        Ok(Self {
            app,
            step_paths: config
                .paths
                .steps
                .into_iter()
                .map(|path| path.into_std_path_buf())
                .collect(),
            scenarios,
            artifact_dir: config.paths.artifacts.into_std_path_buf(),
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
    let context = ExecutionContext::new(run.artifact_dir.clone());
    let mut engine = bootstrap_engine().await?;

    startup(&mut engine, &run.app).await?;

    let patterns = PlaywrightEngine::load_custom_step_patterns(&run.step_paths)
        .await
        .map_err(|error| RunError::Crash(error.to_string()))?;
    let registry = Registry::with_custom_patterns(patterns);
    let mut summary = RunSummary::success();

    for scenario in run.scenarios {
        match execute_scenario(&mut engine, &registry, &run.step_paths, &context, &scenario).await {
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
                summary.output.push(error.to_string());
            }
            Err(error) => {
                let _ = engine.shutdown().await;
                return Err(error);
            }
        }
    }

    engine.shutdown().await.map_err(|error| RunError::Crash(error.to_string()))?;
    Ok(summary)
}

async fn execute_scenario(
    engine: &mut PlaywrightEngine,
    registry: &Registry,
    step_paths: &[PathBuf],
    _context: &ExecutionContext,
    scenario: &CompiledScenario,
) -> Result<Vec<String>, RunError> {
    let mut outputs = Vec::new();

    for step in &scenario.steps {
        let resolved = registry
            .resolve(&step.text)
            .ok_or_else(|| RunError::MissingStep(step.text.clone()))?;

        match resolved.action_name() {
            "click" => {
                let target = resolved.target().ok_or_else(|| {
                    RunError::Crash(format!("missing click target for step: {}", step.text))
                })?;
                engine
                    .click(normalize_target(target.clone()))
                    .await
                    .map_err(|error| RunError::classify(error.to_string()))?;
            }
            "switch_window" => {
                let target = resolved.window_target().ok_or_else(|| {
                    RunError::Crash(format!("missing window target for step: {}", step.text))
                })?;
                let description = engine
                    .switch_window(target.clone())
                    .await
                    .map_err(|error| RunError::classify(error.to_string()))?;
                outputs.push(description);
            }
            "custom" => {
                let app_title = engine
                    .current_window_title()
                    .await
                    .map_err(|error| RunError::classify(error.to_string()))?;
                let output = PlaywrightEngine::execute_custom_step(step_paths, &step.text, &app_title)
                    .await
                    .map_err(|error| RunError::classify(error.to_string()))?;
                if !output.is_empty() {
                    outputs.push(output);
                }
            }
            other => return Err(RunError::Crash(format!("unsupported step action: {other}"))),
        }
    }

    Ok(outputs)
}

async fn bootstrap_engine() -> Result<PlaywrightEngine, RunError> {
    let cache_dir = crate::project::bootstrap::ensure_worker_runtime()
        .await
        .map_err(|error| RunError::Crash(error.to_string()))?;
    let runtime = crate::project::bootstrap::materialize_runtime(cache_dir.as_std_path())
        .await
        .map_err(|error| RunError::Crash(error.to_string()))?;
    let mut command = tokio::process::Command::new("node");
    command.arg(runtime.join("index.js").as_str());
    let worker = WorkerProcess::from_command(command)
        .map_err(|error: crate::engine::WorkerProcessError| RunError::Crash(error.to_string()))?;
    Ok(PlaywrightEngine::new(worker))
}

async fn startup(engine: &mut PlaywrightEngine, app: &AppRequest) -> Result<(), RunError> {
    match app {
        AppRequest::Attach { endpoint } => engine
            .attach(endpoint)
            .await
            .map(|_| ())
            .map_err(|error| RunError::classify(error.to_string())),
        AppRequest::Launch { command, args } => engine
            .launch(command, args)
            .await
            .map(|_| ())
            .map_err(|error| RunError::classify(error.to_string())),
    }
}

async fn resolve_attach_endpoint(config: &Config) -> Result<String, RunError> {
    if let Some(endpoint) = &config.app.endpoint {
        return Ok(endpoint.clone());
    }

    let endpoint_file = config
        .app
        .endpoint_file
        .as_ref()
        .ok_or_else(|| RunError::Config("missing attach endpoint".into()))?;
    let endpoint = tokio::fs::read_to_string(endpoint_file)
        .await
        .map_err(|error| RunError::Crash(format!("failed to read attach endpoint file: {error}")))?;
    Ok(endpoint.trim().to_owned())
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
