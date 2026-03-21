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
