#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::sync::{Mutex, OnceLock};

use assert_cmd::Command;

fn fixture_install_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub async fn fixture_project() -> FixtureProject {
    FixtureProject::from_repo_paths("tests/fixtures").await
}

pub struct FixtureProject {
    pub root: PathBuf,
}

impl FixtureProject {
    async fn from_repo_paths(root: &str) -> Self {
        Self {
            root: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(root),
        }
    }
}

pub struct FixtureRun {
    pub status: ExitStatus,
    pub stdout: String,
    pub artifact_dir: PathBuf,
    pub app_root: Option<PathBuf>,
}

pub struct PreparedAttachFixture {
    pub project_root: PathBuf,
    endpoint_file: PathBuf,
    artifact_dir: PathBuf,
    launcher_script: PathBuf,
    fixture_app_root: PathBuf,
}

pub async fn run_fixture(feature_name: &str) -> FixtureRun {
    ensure_fixture_dependencies().await;
    run_launch_fixture(feature_name).await
}

pub async fn run_attach_fixture(feature_name: &str) -> FixtureRun {
    ensure_fixture_dependencies().await;
    let prepared = prepare_attach_fixture_project(feature_name).await;
    run_prepared_attach_fixture(prepared).await
}

pub async fn run_with_config(raw_config: &str) -> FixtureRun {
    ensure_fixture_dependencies().await;
    run_electrotest_project(raw_config, None, None, None).await
}

pub fn copy_fixture_file(source: &Path, destination: &Path) {
    let source = std::fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf());
    let destination = std::fs::canonicalize(destination).unwrap_or_else(|_| destination.to_path_buf());

    if source == destination {
        return;
    }

    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::copy(&source, &destination).unwrap();
}

async fn ensure_fixture_dependencies() {
    let _ = fixture_install_lock();
}

async fn run_launch_fixture(feature_name: &str) -> FixtureRun {
    let fixture = fixture_project().await;
    let fixture_app_root = prepare_fixture_app_root(&fixture.root);
    let electron_bin = fixture_app_root.join("node_modules/.bin/electron");
    let raw_config = format!(
        "[app]\nmode = \"launch\"\ncommand = {:?}\nargs = [{:?}]\n\n[paths]\nfeatures = [\"features/{feature_name}\"]\nsteps = [\"steps/sample.steps.ts\"]\nartifacts = \".electrotest/artifacts\"\n",
        electron_bin.to_string_lossy(),
        fixture_app_root.to_string_lossy(),
    );

    run_electrotest_project(
        &raw_config,
        Some((feature_name, fixture.root.join("features").join(feature_name))),
        Some(fixture.root.join("steps/sample.steps.ts")),
        Some(fixture_app_root),
    )
    .await
}

pub async fn prepare_attach_fixture_project(feature_name: &str) -> PreparedAttachFixture {
    let fixture = fixture_project().await;
    let attach_fixture_root = fixture.root.join("attach");
    let fixture_app_root = prepare_fixture_app_root(&fixture.root);
    let project_root = temp_project_root();
    let project_attach_root = project_root.join("attach");
    let artifact_dir = project_attach_root.join(".electrotest/artifacts");
    let endpoint_dir = project_attach_root.join(".electrotest");
    let endpoint_file = endpoint_dir.join("attach-endpoint.txt");
    let feature_dir = project_root.join("features");
    let step_dir = project_root.join("steps");
    std::fs::create_dir_all(&artifact_dir).unwrap();
    std::fs::create_dir_all(&endpoint_dir).unwrap();
    std::fs::create_dir_all(&project_attach_root).unwrap();
    std::fs::create_dir_all(&feature_dir).unwrap();
    std::fs::create_dir_all(&step_dir).unwrap();

    copy_fixture_file(
        &attach_fixture_root.join("electrotest.toml"),
        &project_attach_root.join("electrotest.toml"),
    );
    copy_fixture_file(
        &fixture.root.join("features").join(feature_name),
        &feature_dir.join(feature_name),
    );
    copy_fixture_file(
        &fixture.root.join("steps/sample.steps.ts"),
        &step_dir.join("sample.steps.ts"),
    );

    PreparedAttachFixture {
        project_root,
        endpoint_file,
        artifact_dir,
        launcher_script: attach_fixture_root.join("start-attached-session.mjs"),
        fixture_app_root,
    }
}

pub async fn run_prepared_attach_fixture(prepared: PreparedAttachFixture) -> FixtureRun {
    let mut child = std::process::Command::new("node")
        .arg(&prepared.launcher_script)
        .arg(&prepared.endpoint_file)
        .arg(&prepared.fixture_app_root)
        .current_dir(workspace_root())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    wait_for_file(&prepared.endpoint_file);
    let endpoint = std::fs::read_to_string(&prepared.endpoint_file).unwrap();
    wait_for_cdp_endpoint(endpoint.trim()).await;

    let result = run_command_in_project(
        &prepared.project_root.join("attach"),
        prepared.artifact_dir.clone(),
        Some(prepared.fixture_app_root.clone()),
    );

    terminate_child(&mut child);
    result
}

async fn run_electrotest_project(
    raw_config: &str,
    feature: Option<(&str, PathBuf)>,
    step_file: Option<PathBuf>,
    app_root: Option<PathBuf>,
) -> FixtureRun {
    let project_root = temp_project_root();
    let artifact_dir = project_root.join(".electrotest/artifacts");
    let feature_dir = project_root.join("features");
    let step_dir = project_root.join("steps");
    std::fs::create_dir_all(&feature_dir).unwrap();
    std::fs::create_dir_all(&step_dir).unwrap();
    std::fs::create_dir_all(&artifact_dir).unwrap();

    if let Some((feature_name, feature_path)) = feature {
        copy_fixture_file(&feature_path, &feature_dir.join(feature_name));
    }
    if let Some(step_file) = step_file {
        copy_fixture_file(&step_file, &step_dir.join("sample.steps.ts"));
    }

    std::fs::write(project_root.join("electrotest.toml"), raw_config).unwrap();
    run_command_in_project(&project_root, artifact_dir, app_root)
}

fn run_command_in_project(project_root: &Path, artifact_dir: PathBuf, app_root: Option<PathBuf>) -> FixtureRun {
    let assert = Command::cargo_bin("electrotest")
        .unwrap()
        .current_dir(project_root)
        .arg("test")
        .assert();
    fixture_run_from_assert(assert.get_output(), artifact_dir, app_root)
}

fn fixture_run_from_assert(
    output: &std::process::Output,
    artifact_dir: PathBuf,
    app_root: Option<PathBuf>,
) -> FixtureRun {
    let status = output.status;
    let mut stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        if !stdout.is_empty() {
            stdout.push('\n');
        }
        stdout.push_str(&stderr);
    }

    FixtureRun {
        status,
        stdout,
        artifact_dir,
        app_root,
    }
}

fn install_npm_dependencies(dir: &Path) {
    let package_json = dir.join("package.json");
    if !package_json.exists() {
        return;
    }

    let node_modules = dir.join("node_modules");
    if node_modules.exists() {
        return;
    }

    let status = std::process::Command::new("npm")
        .arg("install")
        .current_dir(dir)
        .status()
        .unwrap();
    assert!(status.success(), "npm install failed in {}", dir.display());
}

fn prepare_fixture_app_root(fixture_root: &Path) -> PathBuf {
    let _lock = fixture_install_lock().lock().unwrap();
    let app_root = temp_project_root().join("electron-app");
    copy_fixture_directory(
        &fixture_root.join("electron-app"),
        &app_root,
        &["node_modules", "package-lock.json"],
    );
    install_npm_dependencies(&app_root);
    app_root
}

fn copy_fixture_directory(source: &Path, destination: &Path, exclude_names: &[&str]) {
    std::fs::create_dir_all(destination).unwrap();

    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if exclude_names.iter().any(|excluded| *excluded == name) {
            continue;
        }

        let dest_path = destination.join(&file_name);
        if path.is_dir() {
            copy_fixture_directory(&path, &dest_path, exclude_names);
        } else {
            copy_fixture_file(&path, &dest_path);
        }
    }
}

fn wait_for_file(path: &Path) {
    for _ in 0..100 {
        if path.exists() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("timed out waiting for file: {}", path.display());
}

fn terminate_child(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let _ = std::process::Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
            .status();

        for _ in 0..20 {
            if child.try_wait().unwrap().is_some() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    let _ = child.kill();
    let _ = child.wait();
}

async fn wait_for_cdp_endpoint(endpoint: &str) {
    let url = format!("{}/json/version", endpoint.trim_end_matches('/'));
    for _ in 0..100 {
        let status = tokio::process::Command::new("node")
            .arg("-e")
            .arg(format!(
                "fetch({url:?}).then((res) => process.exit(res.ok ? 0 : 1)).catch(() => process.exit(1))"
            ))
            .status()
            .await
            .unwrap();
        if status.success() {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("timed out waiting for CDP endpoint: {endpoint}");
}

fn fixture_root() -> PathBuf {
    workspace_root().join("tests/fixtures")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn temp_project_root() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "electrotest-fixture-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    root
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn fixture_app_preparation_waits_for_install_lock() {
        let temp = tempfile::tempdir().unwrap();
        let fixture_root = temp.path();
        let electron_app = fixture_root.join("electron-app");
        std::fs::create_dir_all(&electron_app).unwrap();
        std::fs::write(electron_app.join("main.js"), "console.log('fixture');\n").unwrap();

        let lock = fixture_install_lock().lock().unwrap();
        let fixture_root = fixture_root.to_path_buf();
        let (tx, rx) = mpsc::channel();

        let handle = std::thread::spawn(move || {
            let app_root = prepare_fixture_app_root(&fixture_root);
            tx.send(app_root).unwrap();
        });

        assert!(
            rx.recv_timeout(Duration::from_millis(200)).is_err(),
            "fixture app preparation should wait while the install lock is held"
        );

        drop(lock);
        handle.join().unwrap();
    }
}
