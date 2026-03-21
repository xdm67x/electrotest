mod support;

#[test]
fn classifies_assertion_failures() {
    let error = electrotest::runner::RunError::classify(
        "execute custom step failed: Error: expected fixture window title to be Missing Window, got Fixture App",
    );

    assert!(matches!(error, electrotest::runner::RunError::Assertion(_)));
}

#[tokio::test]
async fn stores_screenshot_and_trace_when_step_fails() {
    let result = support::run_fixture("failing-assertion.feature").await;

    assert!(result.artifact_dir.join("failure.png").exists());
    assert!(result.artifact_dir.join("trace.zip").exists());
}
