use assert_fs::TempDir;
use novalyn_core::config::{LoadOptions, load_config};
use novalyn_core::git::RawCommit;
use novalyn_core::parse::{BumpKind, infer_version, parse_and_classify};
use semver::Version;

/// Create a RawCommit with the given summary for version inference tests.
fn mk(summary: &str) -> RawCommit {
    RawCommit {
        id: "x".into(),
        short_id: "x".into(),
        summary: summary.into(),
        body: String::new().into(),
        author_name: "A".into(),
        author_email: "a@b.c".into(),
        timestamp: 0,
    }
}

#[test]
fn bump_rules_pre_1() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();
    let commits = parse_and_classify(vec![mk("feat: add"), mk("fix: bug")].into(), &cfg);
    let (new, kind) = infer_version(&Version::parse("0.1.0").unwrap(), &commits, None);
    assert_eq!(new, Version::parse("0.1.1").unwrap());
    assert_eq!(
        kind,
        BumpKind::Patch,
        "expected patch bump kind under pre-1.0 rule"
    );
}

#[test]
fn bump_rules_breaking_pre_1() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();
    let commits = parse_and_classify(vec![mk("feat!: change")].into(), &cfg);
    let (new, kind) = infer_version(&Version::parse("0.1.0").unwrap(), &commits, None);
    assert_eq!(new, Version::parse("0.2.0").unwrap());
    assert_eq!(kind, BumpKind::Major);
}

/// Test version bump rules for normal (>=1.0.0) versions.
#[test]
fn bump_rules_normal() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();
    let commits = parse_and_classify(vec![mk("feat: add"), mk("fix: bug")].into(), &cfg);
    let (new, kind) = infer_version(&Version::parse("1.1.0").unwrap(), &commits, None);
    assert_eq!(new, Version::parse("1.2.0").unwrap());
    assert_eq!(kind, BumpKind::Minor);
}
