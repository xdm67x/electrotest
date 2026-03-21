use std::fs;

struct CurrentDirGuard(std::path::PathBuf);

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.0).unwrap();
    }
}

#[test]
fn parses_launch_mode_config() {
    let raw = r#"
        [app]
        mode = "launch"
        command = "npm"
        args = ["run", "start"]

        [paths]
        features = ["features"]
        steps = ["steps"]
        artifacts = ".electrotest/artifacts"
    "#;

    let config = electrotest::config::from_str(raw).unwrap();
    assert_eq!(config.app.mode.as_str(), "launch");
}

#[test]
fn rejects_launch_mode_without_command() {
    let raw = r#"
        [app]
        mode = "launch"

        [paths]
        features = ["features"]
        steps = ["steps"]
        artifacts = ".electrotest/artifacts"
    "#;

    let error = electrotest::config::from_str(raw).unwrap_err();
    assert!(matches!(
        error,
        electrotest::config::ConfigError::MissingLaunchCommand
    ));
}

#[test]
fn parses_attach_mode_with_endpoint_file() {
    let raw = r#"
        [app]
        mode = "attach"
        endpoint_file = ".electrotest/attach-endpoint.txt"

        [paths]
        features = ["features"]
        steps = ["steps"]
        artifacts = ".electrotest/artifacts"
    "#;

    let config = electrotest::config::from_str(raw).unwrap();
    assert_eq!(config.app.mode.as_str(), "attach");
    assert_eq!(
        config.app.endpoint_file.unwrap().as_str(),
        ".electrotest/attach-endpoint.txt"
    );
}

#[test]
fn parses_attach_mode_with_endpoint() {
    let raw = r#"
        [app]
        mode = "attach"
        endpoint = "ws://127.0.0.1:9222/devtools/browser/123"

        [paths]
        features = ["features"]
        steps = ["steps"]
        artifacts = ".electrotest/artifacts"
    "#;

    let config = electrotest::config::from_str(raw).unwrap();
    assert_eq!(config.app.mode.as_str(), "attach");
    assert_eq!(
        config.app.endpoint.as_deref(),
        Some("ws://127.0.0.1:9222/devtools/browser/123")
    );
}

#[test]
fn rejects_attach_mode_without_endpoint_source() {
    let raw = r#"
        [app]
        mode = "attach"

        [paths]
        features = ["features"]
        steps = ["steps"]
        artifacts = ".electrotest/artifacts"
    "#;

    let error = electrotest::config::from_str(raw).unwrap_err();
    assert!(matches!(
        error,
        electrotest::config::ConfigError::MissingAttachEndpoint
    ));
}

#[test]
fn rejects_attach_mode_with_conflicting_endpoint_sources() {
    let raw = r#"
        [app]
        mode = "attach"
        endpoint = "ws://127.0.0.1:9222/devtools/browser/123"
        endpoint_file = ".electrotest/attach-endpoint.txt"

        [paths]
        features = ["features"]
        steps = ["steps"]
        artifacts = ".electrotest/artifacts"
    "#;

    let error = electrotest::config::from_str(raw).unwrap_err();
    assert!(matches!(
        error,
        electrotest::config::ConfigError::ConflictingAttachEndpointSources
    ));
}

#[test]
fn resolves_config_paths_relative_to_loaded_file() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    fs::create_dir_all(&root).unwrap();

    let config_path = root.join("electrotest.toml");
    fs::write(
        &config_path,
        r#"
            [app]
            mode = "attach"
            endpoint_file = ".electrotest/attach-endpoint.txt"

            [paths]
            features = ["features"]
            steps = ["steps"]
            artifacts = ".electrotest/artifacts"
        "#,
    )
    .unwrap();

    let _current_dir_guard = CurrentDirGuard(std::env::current_dir().unwrap());
    let other_dir = temp.path().join("elsewhere");
    fs::create_dir_all(&other_dir).unwrap();
    std::env::set_current_dir(&other_dir).unwrap();

    let config = electrotest::config::from_path(&config_path).unwrap();

    assert_eq!(
        config.paths.features[0].as_std_path(),
        root.join("features")
    );
    assert_eq!(config.paths.steps[0].as_std_path(), root.join("steps"));
    assert_eq!(
        config.paths.artifacts.as_std_path(),
        root.join(".electrotest/artifacts")
    );
    assert_eq!(
        config.app.endpoint_file.unwrap().as_std_path(),
        root.join(".electrotest/attach-endpoint.txt")
    );
}
