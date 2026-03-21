mod support;

use std::fs;

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

#[test]
fn fixture_support_skips_same_path_copy_to_protect_source_files() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("fixture.feature");
    fs::write(&path, "Feature: Guard\n").unwrap();

    support::copy_fixture_file(&path, &path);

    assert_eq!(fs::read_to_string(&path).unwrap(), "Feature: Guard\n");
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
