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
    assert!(main_js.contains("Settings Window"));
    assert!(main_js.contains("document.getElementById(\"open-settings\").addEventListener(\"click\", () => {"));
    assert!(main_js.contains("createSettingsWindow();"));
    assert!(!main_js.contains("app.whenReady().then(() => {\n  createWindow(\"Fixture App\");\n  createSettingsWindow();"));

    let sample_steps = fs::read_to_string(fixture.root.join("steps/sample.steps.ts")).unwrap();
    assert!(sample_steps.contains("the fixture window title should be {string}"));
}

#[tokio::test]
async fn executes_custom_typescript_step() {
    let result = support::run_fixture("custom-step.feature").await;
    assert!(result.stdout.contains("custom step executed"));
}
