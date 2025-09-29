use changelogen::git::add_and_commit;
use changelogen::pipeline::{ExitCode, ReleaseOptions, run_release};
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
fn dry_run_leaves_changelog_untouched() {
    let (td, repo) = init_repo();
    std::fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&repo, "feat: one").unwrap();
    let outcome = run_release(ReleaseOptions {
        cwd: td.path().into(),
        from: None,
        to: None,
        dry_run: true,
        new_version: None,
        no_authors: true,
        exclude_authors: vec![],
        hide_author_email: false,
        clean: false,
        sign: false,
        yes: true,
    })
    .unwrap();
    assert_eq!(outcome.exit as i32, ExitCode::NoChange as i32); // dry run reports no change (wrote=false)
    assert!(!outcome.changelog_path.exists());
}

#[test]
fn exit_code_no_change() {
    let (td, repo) = init_repo();
    std::fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&repo, "feat: one").unwrap();
    // First real run
    let outcome1 = run_release(ReleaseOptions {
        cwd: td.path().into(),
        from: None,
        to: None,
        dry_run: false,
        new_version: None,
        no_authors: true,
        exclude_authors: vec![],
        hide_author_email: false,
        clean: false,
        sign: false,
        yes: true,
    })
    .unwrap();
    assert!(outcome1.wrote);
    assert_eq!(outcome1.exit as i32, ExitCode::Success as i32);
    // Second run with no new commits
    let outcome2 = run_release(ReleaseOptions {
        cwd: td.path().into(),
        from: None,
        to: None,
        dry_run: false,
        new_version: None,
        no_authors: true,
        exclude_authors: vec![],
        hide_author_email: false,
        clean: false,
        sign: false,
        yes: true,
    })
    .unwrap();
    assert!(!outcome2.wrote);
    assert_eq!(outcome2.version, outcome1.version); // unchanged version
    assert_eq!(outcome2.exit as i32, ExitCode::NoChange as i32);
}
