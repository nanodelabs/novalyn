use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use predicates::prelude::*;

#[test]
fn cli_generate_help() {
    let mut cmd = cargo_bin_cmd!("novalyn");
    cmd.arg("generate").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn cli_release_help() {
    let mut cmd = cargo_bin_cmd!("novalyn");
    cmd.arg("release").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn cli_github_help() {
    let mut cmd = cargo_bin_cmd!("novalyn");
    cmd.arg("github").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn cli_show_help() {
    let mut cmd = cargo_bin_cmd!("novalyn");
    cmd.arg("show").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn cli_completions_help() {
    let mut cmd = cargo_bin_cmd!("novalyn");
    cmd.arg("completions").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}

#[test]
fn cli_version_flag() {
    let mut cmd = cargo_bin_cmd!("novalyn");
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("novalyn"));
}

#[test]
fn cli_generate_no_git_repo() {
    let temp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("novalyn");
    cmd.current_dir(temp.path());
    cmd.arg("generate");
    cmd.assert().failure();
}

#[test]
fn cli_release_no_git_repo() {
    let temp = TempDir::new().unwrap();
    let mut cmd = cargo_bin_cmd!("novalyn");
    cmd.current_dir(temp.path());
    cmd.arg("release");
    cmd.assert().failure();
}

#[test]
fn cli_shell_completion_bash() {
    let mut cmd = cargo_bin_cmd!("novalyn");
    cmd.arg("completions").arg("bash");
    cmd.assert().success();
}
