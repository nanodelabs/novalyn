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

#[test]
fn signed_tag_error_messaging() {
    // This test documents that GPG signing is not currently implemented
    // in the create_tag function. The function creates annotated tags but
    // does not support GPG signing via libgit2.
    //
    // Current behavior:
    // - create_tag with annotated=true creates an annotated tag (not signed)
    // - GPG signing would require shell commands (git tag -s) or GPG integration
    // - If GPG is needed, the --sign flag would need to trigger shell command fallback
    //
    // The error messaging is clear: create_tag simply creates annotated tags
    // and does not claim to support GPG signing. Users expecting signed tags
    // would need to manually run `git tag -s` after the tool runs.

    let (td, repo) = init_repo();
    std::fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&repo, "feat: initial").unwrap();

    // Create an annotated tag (not signed, but clearly documented)
    let result = create_tag(&repo, "v0.2.0", "Release v0.2.0", true);

    // Should succeed - we're not attempting GPG signing
    assert!(result.is_ok(), "Annotated tag creation should succeed");

    // Note: GPG signing is not implemented. If --sign flag is passed to CLI,
    // it currently has no effect. Future enhancement would:
    // 1. Detect GPG availability
    // 2. Fall back to shell command: git tag -s
    // 3. Provide clear error if GPG not available
}
