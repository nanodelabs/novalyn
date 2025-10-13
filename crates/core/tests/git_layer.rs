use novalyn_core::git::*;
use std::fs;
use tempfile::TempDir;

/// Initialize a temporary git repository for testing git operations.
fn init_repo() -> (TempDir, gix::Repository) {
    let td = TempDir::new().unwrap();
    let repo = novalyn_core::git::init_repo(td.path()).unwrap();
    (td, repo)
}

#[test]
fn detect_and_initial_commit() {
    let (td, mut repo) = init_repo();
    // no commits yet
    assert!(current_ref(&repo).unwrap().is_none());
    // create file
    let file_path = td.path().join("README.md");
    fs::write(&file_path, "Hello").unwrap();
    add_and_commit(&mut repo, "chore: initial").unwrap();
    assert!(current_ref(&repo).unwrap().is_some());
    assert!(!is_dirty(&repo).unwrap());
}

#[test]
fn tag_discovery_and_ordering() {
    let (td, mut repo) = init_repo();
    // commit 1
    fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&mut repo, "feat: one").unwrap();
    create_tag(&mut repo, "v0.1.0", "v0.1.0", true).unwrap();
    // commit 2
    fs::write(td.path().join("b.txt"), "2").unwrap();
    add_and_commit(&mut repo, "feat: two").unwrap();
    create_tag(&mut repo, "v0.2.0", "v0.2.0", false).unwrap();
    let last = last_tag(&repo).unwrap();
    assert_eq!(last.as_deref(), Some("v0.2.0"));
}

#[test]
fn commits_between_works() {
    let (td, mut repo) = init_repo();
    fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&mut repo, "feat: one").unwrap();
    create_tag(&mut repo, "v0.1.0", "v0.1.0", true).unwrap();
    fs::write(td.path().join("b.txt"), "2").unwrap();
    add_and_commit(&mut repo, "feat: two\n\nbody line").unwrap();
    let head = repo.head().unwrap().id().unwrap().to_string();
    let commits = commits_between(&repo, Some("v0.1.0"), &head).unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].summary, "feat: two");
    assert_eq!(commits[0].body.trim(), "body line");
}

/// Test dirty detection with untracked files in the repository.
#[test]
fn dirty_detection_with_untracked() {
    let (td, mut repo) = init_repo();
    fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&mut repo, "feat: one").unwrap();
    // untracked file
    fs::write(td.path().join("u.txt"), "u").unwrap();
    assert!(is_dirty(&repo).unwrap());
    assert!(is_dirty(&repo).unwrap());
}
