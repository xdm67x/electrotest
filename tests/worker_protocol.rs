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
    let cache = tempfile::tempdir().unwrap();
    let runtime = electrotest::project::bootstrap::materialize_runtime(cache.path())
        .await
        .unwrap();

    assert!(runtime.join("index.js").exists());
}

#[tokio::test]
async fn starts_bootstrapped_worker_and_exchanges_ping() {
    let cache = tempfile::tempdir().unwrap();
    let runtime = electrotest::project::bootstrap::materialize_runtime(cache.path())
        .await
        .unwrap();
    let runtime_root = runtime.parent().unwrap();

    assert!(runtime_root.join("node_modules/playwright").exists());

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
