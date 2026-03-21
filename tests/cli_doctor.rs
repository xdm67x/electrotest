use assert_cmd::Command;

fn write_fake_node(dir: &std::path::Path, version: &str) {
    let node_path = dir.join("node");
    std::fs::write(
        &node_path,
        format!("#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  printf '%s\\n' '{version}'\n  exit 0\nfi\nexit 0\n"),
    )
    .unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = std::fs::metadata(&node_path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&node_path, permissions).unwrap();
    }
}

fn write_launch_config(dir: &std::path::Path, command_line: &str, features: &str, steps: &str) {
    std::fs::write(
        dir.join("electrotest.toml"),
        format!(
            "[app]\nmode = \"launch\"\n{command_line}\n\n[paths]\nfeatures = [\"{features}\"]\nsteps = [\"{steps}\"]\nartifacts = \".electrotest/artifacts\"\n"
        ),
    )
    .unwrap();
}

fn write_worker_dependency(cache_dir: &std::path::Path) {
    let runtime_dir = cache_dir.join("worker/v1");
    std::fs::create_dir_all(runtime_dir.join("node_modules/playwright")).unwrap();
    std::fs::write(runtime_dir.join("package.json"), "{}\n").unwrap();
}

#[test]
fn doctor_fails_when_node_is_missing() {
    let temp = tempfile::tempdir().unwrap();

    Command::cargo_bin("electrotest")
        .unwrap()
        .current_dir(temp.path())
        .env("PATH", "")
        .arg("doctor")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Node.js"));
}

#[test]
fn doctor_fails_when_node_version_is_unsupported() {
    let temp = tempfile::tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    write_fake_node(&bin_dir, "v16.0.0");

    Command::cargo_bin("electrotest")
        .unwrap()
        .current_dir(temp.path())
        .env("PATH", &bin_dir)
        .arg("doctor")
        .assert()
        .failure()
        .stderr(predicates::str::contains("supported"));
}

#[test]
fn doctor_fails_when_worker_dependency_is_missing() {
    let temp = tempfile::tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    let cache_dir = temp.path().join("cache");
    std::fs::create_dir_all(&bin_dir).unwrap();
    std::fs::create_dir_all(cache_dir.join("worker/v1")).unwrap();
    write_fake_node(&bin_dir, "v20.0.0");
    std::fs::write(cache_dir.join("worker/v1/package.json"), "{}\n").unwrap();

    Command::cargo_bin("electrotest")
        .unwrap()
        .current_dir(temp.path())
        .env("PATH", &bin_dir)
        .env("ELECTROTEST_CACHE_DIR", &cache_dir)
        .arg("doctor")
        .assert()
        .failure()
        .stderr(predicates::str::contains("playwright"));
}

#[test]
fn doctor_fails_for_invalid_startup_config() {
    let temp = tempfile::tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    let cache_dir = temp.path().join("cache");
    std::fs::create_dir_all(&bin_dir).unwrap();
    write_fake_node(&bin_dir, "v20.0.0");
    write_launch_config(temp.path(), "", "features", "steps");
    std::fs::create_dir_all(temp.path().join("features")).unwrap();
    std::fs::create_dir_all(temp.path().join("steps")).unwrap();

    Command::cargo_bin("electrotest")
        .unwrap()
        .current_dir(temp.path())
        .env("PATH", &bin_dir)
        .env("ELECTROTEST_CACHE_DIR", &cache_dir)
        .arg("doctor")
        .assert()
        .failure()
        .stderr(predicates::str::contains("launch command"));
}

#[test]
fn doctor_succeeds_with_only_ok_output() {
    let temp = tempfile::tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    let cache_dir = temp.path().join("cache");
    std::fs::create_dir_all(&bin_dir).unwrap();
    write_fake_node(&bin_dir, "v20.0.0");
    write_worker_dependency(&cache_dir);
    write_launch_config(temp.path(), "command = \"npm\"", "features", "steps");
    std::fs::create_dir_all(temp.path().join("features")).unwrap();
    std::fs::create_dir_all(temp.path().join("steps")).unwrap();

    Command::cargo_bin("electrotest")
        .unwrap()
        .current_dir(temp.path())
        .env("PATH", &bin_dir)
        .env("ELECTROTEST_CACHE_DIR", &cache_dir)
        .arg("doctor")
        .assert()
        .success()
        .stdout("doctor: ok\n")
        .stderr("");
}

#[test]
fn doctor_reports_missing_feature_path() {
    let temp = tempfile::tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    let cache_dir = temp.path().join("cache");
    std::fs::create_dir_all(&bin_dir).unwrap();
    write_fake_node(&bin_dir, "v20.0.0");
    write_launch_config(
        temp.path(),
        "command = \"npm\"",
        "missing-features",
        "steps",
    );
    std::fs::create_dir_all(temp.path().join("steps")).unwrap();

    Command::cargo_bin("electrotest")
        .unwrap()
        .current_dir(temp.path())
        .env("PATH", &bin_dir)
        .env("ELECTROTEST_CACHE_DIR", &cache_dir)
        .arg("doctor")
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing-features"));
}

#[test]
fn doctor_reports_missing_step_path() {
    let temp = tempfile::tempdir().unwrap();
    let bin_dir = temp.path().join("bin");
    let cache_dir = temp.path().join("cache");
    std::fs::create_dir_all(&bin_dir).unwrap();
    write_fake_node(&bin_dir, "v20.0.0");
    write_launch_config(
        temp.path(),
        "command = \"npm\"",
        "features",
        "missing-steps",
    );
    std::fs::create_dir_all(temp.path().join("features")).unwrap();

    Command::cargo_bin("electrotest")
        .unwrap()
        .current_dir(temp.path())
        .env("PATH", &bin_dir)
        .env("ELECTROTEST_CACHE_DIR", &cache_dir)
        .arg("doctor")
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing-steps"));
}
