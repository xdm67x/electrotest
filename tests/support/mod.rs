#![allow(dead_code)]

use std::path::PathBuf;
use std::process::ExitStatus;

use assert_cmd::Command;

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
}

pub async fn run_fixture(feature_name: &str) -> FixtureRun {
    ensure_fixture_dependencies().await;
    run_electrotest_fixture(feature_name, None).await
}

pub async fn run_attach_fixture(feature_name: &str) -> FixtureRun {
    ensure_fixture_dependencies().await;
    run_electrotest_fixture(feature_name, Some("tests/fixtures/attach/electrotest.toml")).await
}

pub async fn run_with_config(raw_config: &str) -> FixtureRun {
    ensure_fixture_dependencies().await;
    run_electrotest_project(raw_config, None, None).await
}

async fn ensure_fixture_dependencies() {}

async fn run_electrotest_fixture(feature_name: &str, _config_path: Option<&str>) -> FixtureRun {
    let fixture = fixture_project().await;
    let raw_config = format!(
        "[app]\nmode = \"attach\"\nendpoint = \"ws://127.0.0.1:9222/devtools/browser/fixture\"\n\n[paths]\nfeatures = [\"features/{feature_name}\"]\nsteps = [\"steps/sample.steps.ts\"]\nartifacts = \".electrotest/artifacts\"\n"
    );

    run_electrotest_project(&raw_config, Some((feature_name, fixture.root.join("features").join(feature_name))), Some(fixture.root.join("steps/sample.steps.ts"))).await
}

async fn run_electrotest_project(
    raw_config: &str,
    feature: Option<(&str, PathBuf)>,
    step_file: Option<PathBuf>,
) -> FixtureRun {
    let temp = tempfile::tempdir().unwrap();
    let project_root = temp.keep();
    let artifact_dir = project_root.join(".electrotest/artifacts");
    let feature_dir = project_root.join("features");
    let step_dir = project_root.join("steps");
    std::fs::create_dir_all(&feature_dir).unwrap();
    std::fs::create_dir_all(&step_dir).unwrap();
    std::fs::create_dir_all(&artifact_dir).unwrap();

    if let Some((feature_name, feature_path)) = feature {
        std::fs::copy(feature_path, feature_dir.join(feature_name)).unwrap();
    }
    if let Some(step_file) = step_file {
        std::fs::copy(step_file, step_dir.join("sample.steps.ts")).unwrap();
    }

    std::fs::write(project_root.join("electrotest.toml"), raw_config).unwrap();

    let assert = Command::cargo_bin("electrotest")
        .unwrap()
        .current_dir(project_root)
        .arg("test")
        .assert();
    let output = assert.get_output();
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
    }
}
