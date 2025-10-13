use novalyn_core::git::{add_and_commit, create_tag};
use tempfile::TempDir;

/// Initialize a temporary git repository for testing tag creation.
fn init_repo() -> (TempDir, gix::Repository) {
    let td = TempDir::new().unwrap();
    let repo = novalyn_core::git::init_repo(td.path()).unwrap();
    (td, repo)
}

/// Test that annotated git tags are created successfully.
#[test]
fn annotated_tag_creation() {
    let (td, mut repo) = init_repo();
    std::fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&mut repo, "feat: initial").unwrap();
    // annotated true path
    let oid = create_tag(&mut repo, "v0.1.0", "v0.1.0", true).unwrap();
    assert!(!oid.to_string().is_empty());
}
