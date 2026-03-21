use std::path::{Path, PathBuf};

pub struct PlaywrightEngine {
    worker: crate::engine::process::WorkerProcess,
}

impl PlaywrightEngine {
    pub fn new(worker: crate::engine::process::WorkerProcess) -> Self {
        Self { worker }
    }

    pub fn worker(&mut self) -> &mut crate::engine::process::WorkerProcess {
        &mut self.worker
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
