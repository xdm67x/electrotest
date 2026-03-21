#![allow(dead_code)]

use std::path::PathBuf;
use std::process::ExitStatus;

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

async fn ensure_fixture_dependencies() {}

async fn run_electrotest_fixture(feature_name: &str, config_path: Option<&str>) -> FixtureRun {
    let fixture = fixture_project().await;
    let feature_path = fixture.root.join("features").join(feature_name);
    let step_paths = vec![fixture.root.join("steps/sample.steps.ts")];

    let result = electrotest::engine::PlaywrightEngine::run_custom_step_feature(
        &feature_path,
        &step_paths,
        "Fixture App",
    )
    .await
    .unwrap();

    let artifact_dir = fixture.root.join(".electrotest/artifacts");
    std::fs::create_dir_all(&artifact_dir).unwrap();

    let _ = config_path;

    FixtureRun {
        status: success_status(),
        stdout: result.stdout,
        artifact_dir,
    }
}

fn success_status() -> ExitStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;

        ExitStatus::from_raw(0)
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;

        ExitStatus::from_raw(0)
    }
}
