use std::process::Command;
use std::fs;

use changelogen::config::{self, LoadOptions};

fn init_git(dir: &std::path::Path, remote: &str) {
    // Initialize repo via git command (simpler than git2 for author identity config)
    Command::new("git").arg("init").arg(dir).output().unwrap();
    // create an initial commit so git considers repo fully initialized
    fs::write(dir.join("README.md"), "test").unwrap();
    Command::new("git").current_dir(dir).args(["add", "."]).output().unwrap();
    // configure user for commit
    Command::new("git").current_dir(dir).args(["config", "user.email", "you@example.com"]).output().unwrap();
    Command::new("git").current_dir(dir).args(["config", "user.name", "Your Name"]).output().unwrap();
    Command::new("git").current_dir(dir).args(["commit", "-m", "init"]).output().unwrap();
    Command::new("git").current_dir(dir).args(["remote", "add", "origin", remote]).output().unwrap();
}

#[test]
fn detect_repo_origin() {
    let tmp = tempfile::tempdir().unwrap();
    init_git(tmp.path(), "git@github.com:owner/example.git");
    let cfg = config::load_config(LoadOptions { cwd: tmp.path(), cli_overrides: None }).unwrap();
    let repo = cfg.repo.expect("repo detected");
    assert_eq!(repo.owner, "owner");
    assert_eq!(repo.name, "example");
    assert_eq!(format!("{}", repo), "github.com:owner/example");
}

#[test]
fn detect_repo_first_remote_when_no_origin() {
    let tmp = tempfile::tempdir().unwrap();
    // init without origin, then add custom remote name
    std::process::Command::new("git").arg("init").arg(tmp.path()).output().unwrap();
    fs::write(tmp.path().join("lib.rs"), "").unwrap();
    Command::new("git").current_dir(tmp.path()).args(["add", "."]).output().unwrap();
    Command::new("git").current_dir(tmp.path()).args(["config", "user.email", "a@b"]).output().unwrap();
    Command::new("git").current_dir(tmp.path()).args(["config", "user.name", "A"]).output().unwrap();
    Command::new("git").current_dir(tmp.path()).args(["commit", "-m", "init"]).output().unwrap();
    Command::new("git").current_dir(tmp.path()).args(["remote", "add", "upstream", "https://github.com/acme/proj.git"]).output().unwrap();

    let cfg = config::load_config(LoadOptions { cwd: tmp.path(), cli_overrides: None }).unwrap();
    let repo = cfg.repo.expect("repo detected via first remote");
    assert_eq!(repo.owner, "acme");
    assert_eq!(repo.name, "proj");
}