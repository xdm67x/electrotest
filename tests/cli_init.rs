use assert_cmd::Command;
use electrotest::cli::{parse_args, Commands};

#[test]
fn prints_help_for_init_subcommand() {
    Command::cargo_bin("electrotest")
        .unwrap()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage: electrotest init"));
}

#[test]
fn parses_init_subcommand() {
    let cli = parse_args(["electrotest", "init"]);

    assert!(matches!(cli.command(), Some(Commands::Init)));
}
