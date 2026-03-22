mod support;

use std::fs;
use std::process::Command as StdCommand;

#[tokio::test]
async fn fixture_harness_returns_paths_for_test_assets() {
    let fixture = support::fixture_project().await;

    for relative_path in [
        "electron-app/package.json",
        "electron-app/main.js",
        "features/basic-launch.feature",
        "features/multi-window.feature",
        "features/custom-step.feature",
        "features/failing-assertion.feature",
        "features/missing-step.feature",
        "steps/sample.steps.ts",
    ] {
        assert!(
            fixture.root.join(relative_path).exists(),
            "missing fixture asset: {relative_path}"
        );
    }

    let main_js = fs::read_to_string(fixture.root.join("electron-app/main.js")).unwrap();
    assert!(main_js.contains("id=\"launch\""));
    assert!(main_js.contains("id=\"open-settings\""));
    assert!(main_js.contains("Preferences"));
    assert!(main_js.contains("document.getElementById(\"open-settings\").addEventListener(\"click\", () => {"));
    assert!(main_js.contains("createSettingsWindow();"));
    assert!(!main_js.contains("app.whenReady().then(() => {\n  createWindow(\"Fixture App\");\n  createSettingsWindow();"));

    let sample_steps = fs::read_to_string(fixture.root.join("steps/sample.steps.ts")).unwrap();
    assert!(sample_steps.contains("the fixture window title should be {string}"));
}

#[tokio::test]
async fn runs_feature_against_fixture_electron_app() {
    let result = support::run_fixture("basic-launch.feature").await;
    assert!(result.status.success());
    assert!(result.stdout.contains("1 scenario passed"));
}

#[tokio::test]
async fn executes_custom_typescript_step() {
    let result = support::run_fixture("custom-step.feature").await;
    assert!(result.status.success());
    assert!(result.stdout.contains("custom step executed"));
    assert!(result.stdout.contains("1 scenario passed"));
}

#[tokio::test]
async fn attach_mode_can_run_against_existing_fixture_app() {
    let result = support::run_attach_fixture("attach-mode.feature").await;
    assert!(result.status.success());
}

#[tokio::test]
async fn attach_fixture_cleans_up_electron_processes() {
    let result = support::run_attach_fixture("attach-mode.feature").await;

    assert!(result.status.success());

    assert_fixture_electron_process_count(result.app_root.as_ref().unwrap(), 0);
}

#[test]
fn fixture_support_skips_same_path_copy_to_protect_source_files() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("fixture.feature");
    fs::write(&path, "Feature: Guard\n").unwrap();

    support::copy_fixture_file(&path, &path);

    assert_eq!(fs::read_to_string(&path).unwrap(), "Feature: Guard\n");
}

#[tokio::test]
async fn attach_fixture_uses_tracked_fixture_config_layout() {
    let fixture = support::fixture_project().await;
    let config = fs::read_to_string(fixture.root.join("attach/electrotest.toml")).unwrap();
    let prepared = support::prepare_attach_fixture_project("attach-mode.feature").await;

    assert!(config.contains("features = [\"../features\"]"));
    assert!(config.contains("steps = [\"../steps\"]"));
    assert_eq!(fs::read_to_string(prepared.project_root.join("attach/electrotest.toml")).unwrap(), config);

    let result = support::run_prepared_attach_fixture(prepared).await;

    assert!(result.status.success());
}

#[tokio::test]
async fn switches_window_by_title_in_multi_window_scenario() {
    let result = support::run_fixture("multi-window.feature").await;
    assert!(result.status.success());
    assert!(result.stdout.contains("Switched to window: Preferences"));
}

#[tokio::test]
async fn switches_window_by_index_in_multi_window_scenario() {
    let result = support::run_fixture("multi-window.feature").await;
    assert!(result.status.success());
    assert!(result.stdout.contains("Switched to window index 1"));
}

#[tokio::test]
async fn reports_clear_error_when_window_target_is_missing() {
    let result = support::run_fixture("missing-window.feature").await;
    assert!(!result.status.success());
    assert!(result.stdout.contains("window target not found"));
}

#[tokio::test]
async fn reports_clear_error_when_window_target_is_ambiguous() {
    let result = support::run_fixture("ambiguous-window.feature").await;
    assert!(!result.status.success());
    assert!(result.stdout.contains("window target is ambiguous"));
}

#[tokio::test]
async fn test_command_returns_non_zero_on_failure() {
    let result = support::run_fixture("failing-assertion.feature").await;

    assert!(!result.status.success());
    assert!(result.stdout.contains("1 scenario failed"));
}

#[tokio::test]
async fn test_command_normalizes_config_failures_into_runner_errors() {
    let result = support::run_with_config(
        "[app]\nmode = \"launch\"\n\n[paths]\nfeatures = [\"features\"]\nsteps = [\"steps\"]\nartifacts = \".electrotest/artifacts\"\n",
    )
    .await;

    assert!(!result.status.success());
    assert!(result.stdout.contains("config error: missing launch command"));
}

#[tokio::test]
async fn launch_fixture_cleans_up_electron_processes() {
    let result = support::run_fixture("basic-launch.feature").await;

    assert!(result.status.success());

    assert_fixture_electron_process_count(result.app_root.as_ref().unwrap(), 0);
}

#[tokio::test]
async fn fixture_runs_do_not_write_dependency_artifacts_into_tracked_fixtures() {
    let fixture = support::fixture_project().await;
    let electron_node_modules = fixture.root.join("electron-app/node_modules");
    let electron_lockfile = fixture.root.join("electron-app/package-lock.json");
    let attach_lockfile = fixture.root.join("attach/package-lock.json");

    std::fs::remove_dir_all(&electron_node_modules).ok();
    std::fs::remove_file(&electron_lockfile).ok();
    std::fs::remove_file(&attach_lockfile).ok();

    let result = support::run_fixture("basic-launch.feature").await;
    assert!(result.status.success());

    assert!(!electron_node_modules.exists(), "tracked fixture node_modules should not be created");
    assert!(!electron_lockfile.exists(), "tracked fixture package-lock.json should not be created");
    assert!(!attach_lockfile.exists(), "tracked attach package-lock.json should not be created");
}

fn assert_fixture_electron_process_count(app_root: &std::path::Path, expected: usize) {
    for _ in 0..50 {
        let count = fixture_electron_process_count(app_root);
        if count == expected {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let actual = fixture_electron_process_count(app_root);
    assert_eq!(actual, expected, "fixture Electron processes leaked for {}", app_root.display());
}

fn fixture_electron_process_count(app_root: &std::path::Path) -> usize {
    let output = StdCommand::new("ps")
        .args(["-Ao", "command"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let app_root = app_root.to_string_lossy();
    stdout
        .lines()
        .filter(|line| {
            line.contains(app_root.as_ref())
                && (line.contains("node_modules/.bin/electron")
                    || line.contains("Electron.app/Contents/MacOS/Electron")
                    || line.contains("Electron Helper"))
        })
        .count()
}
