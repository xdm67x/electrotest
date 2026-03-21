use std::ffi::OsString;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

fn runtime_bootstrap_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvVarGuard {
    key: &'static str,
    original: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
        let original = std::env::var_os(key);
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(value) => unsafe {
                std::env::set_var(self.key, value);
            },
            None => unsafe {
                std::env::remove_var(self.key);
            },
        }
    }
}

struct FakeWorkerEnvironment {
    _lock: std::sync::MutexGuard<'static, ()>,
    _bin_dir: tempfile::TempDir,
    _path_guard: EnvVarGuard,
    _log_guard: EnvVarGuard,
    npm_log: tempfile::NamedTempFile,
}

impl FakeWorkerEnvironment {
    fn install() -> Self {
        let lock = runtime_bootstrap_lock().lock().unwrap();
        let bin_dir = tempfile::tempdir().unwrap();
        let npm_log = tempfile::NamedTempFile::new().unwrap();
        write_fake_npm(bin_dir.path());
        write_fake_node(bin_dir.path());

        let original_path = std::env::var_os("PATH").unwrap_or_default();
        let path = if original_path.is_empty() {
            OsString::from(bin_dir.path())
        } else {
            let mut joined = OsString::from(bin_dir.path());
            joined.push(":");
            joined.push(original_path);
            joined
        };

        Self {
            _lock: lock,
            _bin_dir: bin_dir,
            _path_guard: EnvVarGuard::set("PATH", &path),
            _log_guard: EnvVarGuard::set("ELECTROTEST_NPM_LOG", npm_log.path()),
            npm_log,
        }
    }

    fn install_log(&self) -> String {
        std::fs::read_to_string(self.npm_log.path()).unwrap()
    }
}

fn write_fake_npm(dir: &Path) {
    let npm_path = dir.join("npm");
    std::fs::write(
        &npm_path,
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$ELECTROTEST_NPM_LOG\"\nif [ \"$1\" = \"install\" ]; then\n  mkdir -p node_modules/playwright\n  exit 0\nfi\nif [ \"$1\" = \"run\" ] && [ \"$2\" = \"build\" ]; then\n  mkdir -p dist\n  printf 'process.stdin.setEncoding(\\\"utf8\\\");\\nfor await (const _ of process.stdin) { process.stdout.write(\"{\\\"type\\\":\\\"pong\\\"}\\\\n\"); }\\n' > dist/index.js\n  exit 0\nfi\necho \"unexpected npm args: $*\" >&2\nexit 1\n",
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = std::fs::metadata(&npm_path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&npm_path, permissions).unwrap();
    }
}

fn write_fake_node(dir: &Path) {
    let node_path = dir.join("node");
    std::fs::write(
        &node_path,
        "#!/bin/sh
if [ ! -f \"$1\" ]; then
  echo \"missing worker entrypoint: $1\" >&2
  exit 1
fi
IFS= read -r _
printf '{\"type\":\"pong\"}\\n'
",
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = std::fs::metadata(&node_path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&node_path, permissions).unwrap();
    }
}

#[tokio::test]
async fn serializes_and_reads_worker_response() {
    let request = electrotest::engine::protocol::Request::Ping;
    let json = serde_json::to_string(&request).unwrap();

    let decoded: electrotest::engine::protocol::Request = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded, electrotest::engine::protocol::Request::Ping);
}

#[tokio::test]
async fn serializes_launch_and_attach_requests() {
    let launch = electrotest::engine::protocol::Request::LaunchApp {
        command: "electron".into(),
        args: vec!["./fixture".into()],
    };
    let launch_json = serde_json::to_string(&launch).unwrap();
    let decoded_launch: electrotest::engine::protocol::Request =
        serde_json::from_str(&launch_json).unwrap();

    assert_eq!(decoded_launch, launch);

    let attach = electrotest::engine::protocol::Request::AttachApp {
        endpoint: "http://127.0.0.1:9222".into(),
    };
    let attach_json = serde_json::to_string(&attach).unwrap();
    let decoded_attach: electrotest::engine::protocol::Request =
        serde_json::from_str(&attach_json).unwrap();

    assert_eq!(decoded_attach, attach);
}

#[tokio::test]
async fn bootstraps_worker_runtime_into_cache() {
    let fake_env = FakeWorkerEnvironment::install();
    let cache = tempfile::tempdir().unwrap();
    let runtime = electrotest::project::bootstrap::materialize_runtime(cache.path())
        .await
        .unwrap();
    let runtime_root = runtime.parent().unwrap();
    let install_log = fake_env.install_log();

    assert!(runtime.join("index.js").exists());
    assert!(runtime_root.join("package-lock.json").exists());
    assert!(runtime_root.join("node_modules/playwright").exists());
    assert!(install_log.lines().any(|line| line == "install"));
    assert!(install_log.lines().any(|line| line == "run build"));
}

#[tokio::test]
async fn bootstraps_worker_runtime_with_npm_install_when_lockfile_is_present() {
    let fake_env = FakeWorkerEnvironment::install();
    let cache = tempfile::tempdir().unwrap();
    let runtime = electrotest::project::bootstrap::materialize_runtime(cache.path())
        .await
        .unwrap();
    let install_log = fake_env.install_log();

    assert!(runtime.join("index.js").exists());
    assert!(install_log.lines().any(|line| line == "install"));
    assert!(install_log.lines().any(|line| line == "run build"));
}

#[tokio::test]
async fn starts_bootstrapped_worker_and_exchanges_ping() {
    let fake_env = FakeWorkerEnvironment::install();
    let cache = tempfile::tempdir().unwrap();
    let runtime = electrotest::project::bootstrap::materialize_runtime(cache.path())
        .await
        .unwrap();
    let runtime_root = runtime.parent().unwrap();
    let install_log = fake_env.install_log();

    assert!(runtime_root.join("node_modules/playwright").exists());
    assert!(install_log.lines().any(|line| line == "install"));
    assert!(install_log.lines().any(|line| line == "run build"));

    let mut command = tokio::process::Command::new("node");
    command.arg(runtime.join("index.js").as_str());

    let mut worker = electrotest::engine::process::WorkerProcess::from_command(command).unwrap();
    let response = worker
        .request(&electrotest::engine::protocol::Request::Ping)
        .await
        .unwrap();

    assert_eq!(response, electrotest::engine::protocol::Response::Pong);
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn starts_worker_and_exchanges_ping() {
    let mut command = tokio::process::Command::new("/bin/sh");
    command
        .arg("-c")
        .arg("IFS= read -r _; printf '{\"type\":\"pong\"}\\n'");

    let mut worker = electrotest::engine::process::WorkerProcess::from_command(command).unwrap();
    let response = worker
        .request(&electrotest::engine::protocol::Request::Ping)
        .await
        .unwrap();

    assert_eq!(response, electrotest::engine::protocol::Response::Pong);
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn rejects_malformed_worker_response() {
    let mut command = tokio::process::Command::new("/bin/sh");
    command.arg("-c").arg("printf 'not-json\\n'");

    let mut worker = electrotest::engine::process::WorkerProcess::from_command(command).unwrap();
    let error = worker.read_response().await.unwrap_err();

    assert!(matches!(
        error,
        electrotest::engine::process::WorkerProcessError::MalformedResponse(raw)
        if raw == "not-json"
    ));
    worker.shutdown().await.unwrap();
}

#[tokio::test]
async fn shutdown_kills_worker_that_does_not_exit_on_eof() {
    let mut command = tokio::process::Command::new("/bin/sh");
    command.arg("-c").arg("sleep 30");

    let mut worker = electrotest::engine::process::WorkerProcess::from_command(command).unwrap();

    tokio::time::timeout(std::time::Duration::from_secs(1), worker.shutdown())
        .await
        .unwrap()
        .unwrap();
}

#[test]
fn resolves_custom_step_patterns_in_registry() {
    let _lock = runtime_bootstrap_lock().lock().unwrap();
    let registry = electrotest::steps::Registry::with_custom_patterns(vec![
        "the fixture window title should be {string}".to_owned(),
    ]);

    let step = registry
        .resolve("Given the fixture window title should be \"Fixture App\"")
        .unwrap();

    assert_eq!(step.action_name(), "custom");
}

#[tokio::test]
async fn loads_custom_typescript_step_patterns() {
    let _lock = runtime_bootstrap_lock().lock().unwrap();
    let step_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/steps/sample.steps.ts")];

    let patterns = electrotest::engine::PlaywrightEngine::load_custom_step_patterns(&step_paths)
        .await
        .unwrap();

    assert!(patterns.iter().any(|pattern| pattern == "the fixture window title should be {string}"));
}

#[tokio::test]
async fn executes_custom_typescript_step_handler() {
    let _lock = runtime_bootstrap_lock().lock().unwrap();
    let step_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/steps/sample.steps.ts")];

    let output = electrotest::engine::PlaywrightEngine::execute_custom_step(
        &step_paths,
        "Given the fixture window title should be \"Fixture App\"",
        "Fixture App",
    )
    .await
    .unwrap();

    assert!(output.contains("custom step executed"));
}

#[tokio::test]
async fn executes_custom_feature_file_through_product_path() {
    let _lock = runtime_bootstrap_lock().lock().unwrap();
    let fixture_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let feature_path = fixture_root.join("features/custom-step.feature");
    let step_paths = vec![fixture_root.join("steps/sample.steps.ts")];

    let result = electrotest::engine::PlaywrightEngine::run_custom_step_feature(
        &feature_path,
        &step_paths,
        "Fixture App",
    )
    .await
    .unwrap();

    assert_eq!(result.scenarios_passed, 1);
    assert!(result.succeeded);
    assert!(result.stdout.contains("custom step executed"));
}

#[tokio::test]
async fn executes_custom_javascript_step_module() {
    let _lock = runtime_bootstrap_lock().lock().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let feature_path = temp.path().join("custom-js-step.feature");
    std::fs::write(
        &feature_path,
        "Feature: Custom JS step\n\n  Scenario: Run JS step\n    Given the fixture js step should run\n",
    )
    .unwrap();
    let step_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/steps/sample.steps.js")];

    let result = electrotest::engine::PlaywrightEngine::run_custom_step_feature(
        &feature_path,
        &step_paths,
        "Fixture App",
    )
    .await
    .unwrap();

    assert_eq!(result.scenarios_passed, 1);
    assert!(result.succeeded);
    assert!(result.stdout.contains("custom js step executed"));
}

#[tokio::test]
async fn rejects_feature_with_unsupported_builtin_step() {
    let _lock = runtime_bootstrap_lock().lock().unwrap();
    let temp = tempfile::tempdir().unwrap();
    let feature_path = temp.path().join("unsupported.feature");
    std::fs::write(
        &feature_path,
        "Feature: Unsupported builtin\n\n  Scenario: Uses builtin\n    When I click on \"Settings\"\n",
    )
    .unwrap();
    let step_paths = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/steps/sample.steps.ts")];

    let error = electrotest::engine::PlaywrightEngine::run_custom_step_feature(
        &feature_path,
        &step_paths,
        "Fixture App",
    )
    .await
    .unwrap_err();

    assert!(error.to_string().contains("unsupported non-custom step"));
}
