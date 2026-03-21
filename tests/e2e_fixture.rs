mod support;

#[tokio::test]
async fn fixture_harness_returns_paths_for_test_assets() {
    let fixture = support::fixture_project().await;
    assert!(fixture.root.join("electron-app/package.json").exists());
    assert!(fixture.root.join("features/basic-launch.feature").exists());
}
