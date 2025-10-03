use changelogen::config::LoadOptions;
use changelogen::git::RawCommit;
use changelogen::parse::{BumpKind, infer_version, parse_and_classify};
use semver::Version;

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
fn no_impact_commits_patch_bump() {
    let td = tempfile::tempdir().unwrap();
    let cfg = changelogen::config::load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();
    // Provide commits that classify to None impact (e.g., docs + chore(other))
    let commits = parse_and_classify(
        vec![mk("docs: update docs"), mk("chore(other): maintenance")].into(),
        &cfg,
    );
    let prev = Version::parse("1.2.3").unwrap();
    let (new, kind) = infer_version(&prev, &commits, None);
    assert_eq!(new, Version::parse("1.2.4").unwrap());
    assert_eq!(kind, BumpKind::Patch);
}
