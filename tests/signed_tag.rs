use changelogen::git::{add_and_commit, create_tag};
use tempfile::TempDir;

fn init_repo() -> (TempDir, git2::Repository) {
    let td = TempDir::new().unwrap();
    let repo = git2::Repository::init(td.path()).unwrap();
    let mut cfg = repo.config().unwrap();
    cfg.set_str("user.name", "Tester").unwrap();
    cfg.set_str("user.email", "tester@example.com").unwrap();
    (td, repo)
}

#[test]
fn annotated_tag_creation() {
    let (td, repo) = init_repo();
    std::fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&repo, "feat: initial").unwrap();
    // annotated true path
    let oid = create_tag(&repo, "v0.1.0", "v0.1.0", true).unwrap();
    assert!(!oid.to_string().is_empty());
}
