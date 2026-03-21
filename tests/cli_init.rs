use assert_cmd::Command;

#[test]
fn prints_help_for_top_level_cli() {
    Command::cargo_bin("electrotest")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("electrotest"));
}

#[test]
fn init_creates_project_files() {
    let temp = tempfile::tempdir().unwrap();

    Command::cargo_bin("electrotest")
        .unwrap()
        .args(["init", "--path", temp.path().to_str().unwrap()])
        .assert()
        .success();

    assert!(temp.path().join("electrotest.toml").exists());
    assert!(temp.path().join("features").exists());
    assert!(temp.path().join("steps").exists());
    assert!(temp.path().join("tsconfig.json").exists());
}

#[test]
fn defaults_to_test_command_when_no_subcommand_is_given() {
    let temp = tempfile::tempdir().unwrap();

    Command::cargo_bin("electrotest")
        .unwrap()
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "config error: missing config file: electrotest.toml",
        ));
}
