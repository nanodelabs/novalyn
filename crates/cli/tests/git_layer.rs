use novalyn::git::*;
use std::fs;
use tempfile::TempDir;

fn init_repo() -> (TempDir, git2::Repository) {
    let td = TempDir::new().unwrap();
    let repo = git2::Repository::init(td.path()).unwrap();
    // configure user
    let mut cfg = repo.config().unwrap();
    cfg.set_str("user.name", "Tester").unwrap();
    cfg.set_str("user.email", "tester@example.com").unwrap();
    (td, repo)
}

#[test]
fn detect_and_initial_commit() {
    let (td, repo) = init_repo();
    // no commits yet
    assert!(current_ref(&repo).unwrap().is_none());
    // create file
    let file_path = td.path().join("README.md");
    fs::write(&file_path, "Hello").unwrap();
    add_and_commit(&repo, "chore: initial").unwrap();
    assert!(current_ref(&repo).unwrap().is_some());
    assert!(!is_dirty(&repo, true).unwrap());
}

#[test]
fn tag_discovery_and_ordering() {
    let (td, repo) = init_repo();
    // commit 1
    fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&repo, "feat: one").unwrap();
    create_tag(&repo, "v0.1.0", "v0.1.0", true).unwrap();
    // commit 2
    fs::write(td.path().join("b.txt"), "2").unwrap();
    add_and_commit(&repo, "feat: two").unwrap();
    create_tag(&repo, "v0.2.0", "v0.2.0", false).unwrap();
    let last = last_tag(&repo).unwrap();
    assert_eq!(last.as_deref(), Some("v0.2.0"));
}

#[test]
fn commits_between_works() {
    let (td, repo) = init_repo();
    fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&repo, "feat: one").unwrap();
    create_tag(&repo, "v0.1.0", "v0.1.0", true).unwrap();
    fs::write(td.path().join("b.txt"), "2").unwrap();
    add_and_commit(&repo, "feat: two\n\nbody line").unwrap();
    let head = repo.head().unwrap().target().unwrap().to_string();
    let commits = commits_between(&repo, Some("v0.1.0"), &head).unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].summary, "feat: two");
    assert_eq!(commits[0].body.trim(), "body line");
}

#[test]
fn dirty_detection_with_untracked() {
    let (td, repo) = init_repo();
    fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&repo, "feat: one").unwrap();
    // untracked file
    fs::write(td.path().join("u.txt"), "u").unwrap();
    assert!(is_dirty(&repo, true).unwrap());
    assert!(!is_dirty(&repo, false).unwrap());
}
