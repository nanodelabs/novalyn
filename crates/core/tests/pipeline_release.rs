use novalyn_core::git::add_and_commit;
use novalyn_core::pipeline::{ExitCode, ReleaseOptions, run_release};
use tempfile::TempDir;

/// Initialize a temporary git repository for testing purposes.
fn init_repo() -> (TempDir, gix::Repository) {
    let td = TempDir::new().unwrap();
    let repo = novalyn_core::git::init_repo(td.path()).unwrap();
    (td, repo)
}

/// Test that a dry run does not modify the changelog file.
#[test]
fn dry_run_leaves_changelog_untouched() {
    let (td, mut repo) = init_repo();
    std::fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&mut repo, "feat: one").unwrap();
    let outcome = run_release(ReleaseOptions {
        cwd: td.path().into(),
        from: None,
        to: None,
        dry_run: true,
        new_version: None,
        no_authors: true,
        exclude_authors: vec![].into(),
        hide_author_email: false,
        clean: false,
        sign: false,
        yes: true,
        github_alias: false,
        github_token: None,
    })
    .unwrap();
    assert_eq!(outcome.exit as i32, ExitCode::NoChange as i32); // dry run reports no change (wrote=false)
    assert!(!outcome.changelog_path.exists());
}

/// Test that the exit code is correct when no new changes are present.
#[test]
fn exit_code_no_change() {
    let (td, mut repo) = init_repo();
    std::fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&mut repo, "feat: one").unwrap();
    // First real run
    let outcome1 = run_release(ReleaseOptions {
        cwd: td.path().into(),
        from: None,
        to: None,
        dry_run: false,
        new_version: None,
        no_authors: true,
        exclude_authors: vec![].into(),
        hide_author_email: false,
        clean: false,
        sign: false,
        yes: true,
        github_alias: false,
        github_token: None,
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
        exclude_authors: vec![].into(),
        hide_author_email: false,
        clean: false,
        sign: false,
        yes: true,
        github_alias: false,
        github_token: None,
    })
    .unwrap();
    assert!(!outcome2.wrote);
    assert_eq!(outcome2.version, outcome1.version); // unchanged version
    assert_eq!(outcome2.exit as i32, ExitCode::NoChange as i32);
}
