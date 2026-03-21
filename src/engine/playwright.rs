use std::path::{Path, PathBuf};

pub struct PlaywrightEngine {
    worker: crate::engine::process::WorkerProcess,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomStepFeatureResult {
    pub scenarios_passed: usize,
    pub stdout: String,
    pub succeeded: bool,
}

impl PlaywrightEngine {
    pub fn new(worker: crate::engine::process::WorkerProcess) -> Self {
        Self { worker }
    }

    pub fn worker(&mut self) -> &mut crate::engine::process::WorkerProcess {
        &mut self.worker
    }

    pub async fn shutdown(&mut self) -> Result<(), crate::Error> {
        self.worker.shutdown().await.map_err(worker_error)
    }

    pub async fn launch(
        &mut self,
        command: &str,
        args: &[String],
    ) -> Result<String, crate::Error> {
        match self
            .worker
            .request(&crate::engine::protocol::Request::LaunchApp {
                command: command.to_owned(),
                args: args.to_vec(),
            })
            .await
            .map_err(worker_error)?
        {
            crate::engine::protocol::Response::AppLaunched { window_id } => Ok(window_id),
            crate::engine::protocol::Response::Error { message } => {
                Err(std::io::Error::other(message).into())
            }
            other => Err(unexpected_response("launch", other)),
        }
    }

    pub async fn attach(&mut self, endpoint: &str) -> Result<String, crate::Error> {
        match self
            .worker
            .request(&crate::engine::protocol::Request::AttachApp {
                endpoint: endpoint.to_owned(),
            })
            .await
            .map_err(worker_error)?
        {
            crate::engine::protocol::Response::AppAttached { window_id } => Ok(window_id),
            crate::engine::protocol::Response::Error { message } => {
                Err(std::io::Error::other(message).into())
            }
            other => Err(unexpected_response("attach", other)),
        }
    }

    pub async fn click(&mut self, locator: Vec<crate::steps::Locator>) -> Result<(), crate::Error> {
        match self
            .worker
            .request(&crate::engine::protocol::Request::Click {
                window_id: "active".to_owned(),
                locator: locator.into_iter().map(Into::into).collect(),
            })
            .await
            .map_err(worker_error)?
        {
            crate::engine::protocol::Response::Clicked => Ok(()),
            crate::engine::protocol::Response::Error { message } => {
                Err(std::io::Error::other(message).into())
            }
            other => Err(unexpected_response("click", other)),
        }
    }

    pub async fn switch_window(
        &mut self,
        target: crate::steps::WindowTarget,
    ) -> Result<String, crate::Error> {
        match self
            .worker
            .request(&crate::engine::protocol::Request::SwitchWindow {
                target: target.into(),
            })
            .await
            .map_err(worker_error)?
        {
            crate::engine::protocol::Response::WindowSwitched { description, .. } => Ok(description),
            crate::engine::protocol::Response::Error { message } => {
                Err(std::io::Error::other(message).into())
            }
            other => Err(unexpected_response("switch_window", other)),
        }
    }

    pub async fn current_window_title(&mut self) -> Result<String, crate::Error> {
        match self
            .worker
            .request(&crate::engine::protocol::Request::CurrentWindowTitle)
            .await
            .map_err(worker_error)?
        {
            crate::engine::protocol::Response::CurrentWindowTitle { title } => Ok(title),
            crate::engine::protocol::Response::Error { message } => {
                Err(std::io::Error::other(message).into())
            }
            other => Err(unexpected_response("current_window_title", other)),
        }
    }

    pub async fn load_custom_step_patterns(step_paths: &[PathBuf]) -> Result<Vec<String>, crate::Error> {
        let script = runtime_loader_script().await?;
        let payload = serde_json::to_string(&step_paths_as_strings(step_paths))?;
        let output = tokio::process::Command::new("node")
            .arg("--input-type=module")
            .arg("-e")
            .arg(script)
            .arg(payload)
            .output()
            .await?;

        if !output.status.success() {
            return Err(command_error("load custom step patterns", &output.stderr));
        }

        Ok(serde_json::from_slice(&output.stdout)?)
    }

    pub async fn execute_custom_step(
        step_paths: &[PathBuf],
        step_text: &str,
        app_title: &str,
    ) -> Result<String, crate::Error> {
        let script = runtime_executor_script().await?;
        let payload = serde_json::json!({
            "stepPaths": step_paths_as_strings(step_paths),
            "stepText": step_text,
            "appTitle": app_title,
        });
        let output = tokio::process::Command::new("node")
            .arg("--input-type=module")
            .arg("-e")
            .arg(script)
            .arg(payload.to_string())
            .output()
            .await?;

        if !output.status.success() {
            return Err(command_error("execute custom step", &output.stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    pub async fn run_custom_step_feature(
        feature_path: &Path,
        step_paths: &[PathBuf],
        app_title: &str,
    ) -> Result<CustomStepFeatureResult, crate::Error> {
        let feature = std::fs::read_to_string(feature_path)?;
        let compiled = crate::gherkin::compile_str(&feature)?;
        let patterns = Self::load_custom_step_patterns(step_paths).await?;
        let registry = crate::steps::Registry::with_custom_patterns(patterns);

        let mut outputs = Vec::new();
        let mut scenarios_passed = 0;

        for scenario in compiled.scenarios {
            for step in scenario.steps {
                let resolved = registry.resolve(&step.text).ok_or_else(|| {
                    std::io::Error::other(format!("no step matched: {}", step.text))
                })?;

                if resolved.action_name() != "custom" {
                    return Err(std::io::Error::other(format!(
                        "unsupported non-custom step in custom-step feature path: {}",
                        step.text,
                    ))
                    .into());
                }

                let output = Self::execute_custom_step(step_paths, &step.text, app_title).await?;
                if !output.is_empty() {
                    outputs.push(output);
                }
            }

            scenarios_passed += 1;
        }

        outputs.push(format!("{scenarios_passed} scenario passed"));

        Ok(CustomStepFeatureResult {
            scenarios_passed,
            stdout: outputs.join("\n"),
            succeeded: true,
        })
    }
}

fn unexpected_response(context: &str, response: crate::engine::protocol::Response) -> crate::Error {
    std::io::Error::other(format!("unexpected {context} response: {response:?}")).into()
}

fn worker_error(error: crate::engine::process::WorkerProcessError) -> crate::Error {
    std::io::Error::other(error.to_string()).into()
}

async fn runtime_loader_script() -> Result<String, crate::Error> {
    let module_path = runtime_steps_module_path().await?;
    Ok(format!(
        "import {{ loadStepModules, registeredStringPatterns }} from {module:?}; const stepPaths = JSON.parse(process.argv[1]); const steps = await loadStepModules(stepPaths); process.stdout.write(JSON.stringify(registeredStringPatterns(steps)));",
        module = module_path,
    ))
}

async fn runtime_executor_script() -> Result<String, crate::Error> {
    let module_path = runtime_steps_module_path().await?;
    Ok(format!(
        "import {{ loadStepModules, findMatchingStep }} from {module:?}; const payload = JSON.parse(process.argv[1]); const steps = await loadStepModules(payload.stepPaths); const match = findMatchingStep(steps, payload.stepText); if (!match) throw new Error(`no custom step matched: ${{payload.stepText}}`); const logs = []; const context = {{ appTitle: async () => payload.appTitle, log: (message) => logs.push(String(message)) }}; await match.step.handler(context, ...match.args); if (logs.length === 0) logs.push('custom step executed'); process.stdout.write(logs.join('\\n'));",
        module = module_path,
    ))
}

async fn runtime_steps_module_path() -> Result<String, crate::Error> {
    let out_dir = transpile_runtime_support().await?;
    let path = out_dir.join("steps.js");
    let path = path.to_str().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "runtime path contains invalid UTF-8")
    })?;
    Ok(path.replace('\\', "\\\\"))
}

async fn transpile_runtime_support() -> Result<PathBuf, crate::Error> {
    let runtime_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("runtime/worker/src");
    let out_dir = unique_temp_dir("electrotest-runtime");
    std::fs::create_dir_all(&out_dir)?;

    let status = tokio::process::Command::new("tsc")
        .args([
            "--target",
            "ES2022",
            "--module",
            "NodeNext",
            "--moduleResolution",
            "NodeNext",
            "--outDir",
        ])
        .arg(&out_dir)
        .arg(runtime_src.join("sdk.ts"))
        .arg(runtime_src.join("steps.ts"))
        .status()
        .await?;

    if status.success() {
        Ok(out_dir)
    } else {
        Err(std::io::Error::other("tsc failed while compiling runtime step support").into())
    }
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()))
}

fn step_paths_as_strings(step_paths: &[PathBuf]) -> Vec<String> {
    step_paths
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect()
}

fn command_error(context: &str, stderr: &[u8]) -> crate::Error {
    std::io::Error::other(format!(
        "{context} failed: {}",
        String::from_utf8_lossy(stderr).trim()
    ))
    .into()
}
