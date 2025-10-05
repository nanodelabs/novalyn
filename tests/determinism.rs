use assert_fs::TempDir;
use novalyn::config::{LoadOptions, load_config};
use novalyn::git::{RawCommit, add_and_commit, create_tag};
use novalyn::parse::parse_and_classify;
use novalyn::pipeline::{ReleaseOptions, run_release};
use novalyn::render::render_release_block;

fn create_test_commits() -> Vec<RawCommit> {
    vec![
        RawCommit {
            id: "abc123".to_string().into(),
            short_id: "abc123".to_string().into(),
            summary: "feat: add new feature".to_string().into(),
            body: "This adds a new feature\n\nCloses #123".to_string().into(),
            author_name: "Alice".to_string().into(),
            author_email: "alice@example.com".to_string().into(),
            timestamp: 1704110400,
        },
        RawCommit {
            id: "def456".to_string().into(),
            short_id: "def456".to_string().into(),
            summary: "fix: resolve bug".to_string().into(),
            body: String::new().into(),
            author_name: "Bob".to_string().into(),
            author_email: "bob@example.com".to_string().into(),
            timestamp: 1704110500,
        },
        RawCommit {
            id: "ghi789".to_string().into(),
            short_id: "ghi789".to_string().into(),
            summary: "feat!: breaking change".to_string().into(),
            body: "BREAKING CHANGE: This breaks API".to_string().into(),
            author_name: "Charlie".to_string().into(),
            author_email: "charlie@example.com".to_string().into(),
            timestamp: 1704110600,
        },
    ]
}

#[test]
fn repeated_parse_identical() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    let commits = create_test_commits();

    // Parse multiple times
    let result1 = parse_and_classify(commits.clone().into(), &cfg);
    let result2 = parse_and_classify(commits.clone().into(), &cfg);
    let result3 = parse_and_classify(commits.into(), &cfg);

    // All results should be identical
    assert_eq!(result1.len(), result2.len());
    assert_eq!(result1.len(), result3.len());

    for ((r1, r2), r3) in result1.iter().zip(result2.iter()).zip(result3.iter()) {
        assert_eq!(r1.raw.id, r2.raw.id);
        assert_eq!(r1.raw.id, r3.raw.id);
        assert_eq!(r1.r#type, r2.r#type);
        assert_eq!(r1.r#type, r3.r#type);
        assert_eq!(r1.scope, r2.scope);
        assert_eq!(r1.scope, r3.scope);
        assert_eq!(r1.description, r2.description);
        assert_eq!(r1.description, r3.description);
        assert_eq!(r1.breaking, r2.breaking);
        assert_eq!(r1.breaking, r3.breaking);
        assert_eq!(r1.index, r2.index);
        assert_eq!(r1.index, r3.index);
    }
}

#[test]
fn repeated_render_identical() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    let commits = create_test_commits();
    let parsed = parse_and_classify(commits.into(), &cfg);

    let version = semver::Version::new(1, 0, 0);
    let prev_version = semver::Version::new(0, 9, 0);

    // Render multiple times
    let ctx1 = novalyn::render::RenderContext {
        commits: &parsed,
        version: &version,
        previous_version: Some(&prev_version),
        authors: None,
        repo: None,
        cfg: &cfg,
        previous_tag: Some("v0.9.0"),
        current_ref: "HEAD",
    };

    let output1 = render_release_block(&ctx1);

    let ctx2 = novalyn::render::RenderContext {
        commits: &parsed,
        version: &version,
        previous_version: Some(&prev_version),
        authors: None,
        repo: None,
        cfg: &cfg,
        previous_tag: Some("v0.9.0"),
        current_ref: "HEAD",
    };

    let output2 = render_release_block(&ctx2);

    // Outputs should be identical
    assert_eq!(output1, output2);
}

#[test]
fn repeated_full_pipeline_identical() {
    let td = TempDir::new().unwrap();
    let repo = git2::Repository::init(td.path()).unwrap();

    // Set up git config
    let mut cfg = repo.config().unwrap();
    cfg.set_str("user.name", "Test").unwrap();
    cfg.set_str("user.email", "test@example.com").unwrap();

    // Create some commits
    std::fs::write(td.path().join("a.txt"), "1").unwrap();
    add_and_commit(&repo, "feat: initial feature").unwrap();

    std::fs::write(td.path().join("b.txt"), "2").unwrap();
    add_and_commit(&repo, "fix: bug fix").unwrap();

    // Tag the first release
    create_tag(&repo, "v0.1.0", "v0.1.0", true).unwrap();

    std::fs::write(td.path().join("c.txt"), "3").unwrap();
    add_and_commit(&repo, "feat: new feature").unwrap();

    // Run release pipeline multiple times with dry-run
    let opts1 = ReleaseOptions {
        cwd: td.path().to_path_buf(),
        from: None,
        to: None,
        dry_run: true,
        new_version: None,
        no_authors: false,
        exclude_authors: vec![].into(),
        hide_author_email: false,
        clean: false,
        sign: false,
        yes: true,
        github_alias: false,
        github_token: None,
    };

    let opts2 = ReleaseOptions {
        cwd: td.path().to_path_buf(),
        from: None,
        to: None,
        dry_run: true,
        new_version: None,
        no_authors: false,
        exclude_authors: vec![].into(),
        hide_author_email: false,
        clean: false,
        sign: false,
        yes: true,
        github_alias: false,
        github_token: None,
    };

    let opts3 = ReleaseOptions {
        cwd: td.path().to_path_buf(),
        from: None,
        to: None,
        dry_run: true,
        new_version: None,
        no_authors: false,
        exclude_authors: vec![].into(),
        hide_author_email: false,
        clean: false,
        sign: false,
        yes: true,
        github_alias: false,
        github_token: None,
    };

    let outcome1 = run_release(opts1).unwrap();
    let outcome2 = run_release(opts2).unwrap();
    let outcome3 = run_release(opts3).unwrap();

    // All outcomes should be identical
    assert_eq!(outcome1.version, outcome2.version);
    assert_eq!(outcome1.version, outcome3.version);
    assert_eq!(outcome1.commit_count, outcome2.commit_count);
    assert_eq!(outcome1.commit_count, outcome3.commit_count);
}
