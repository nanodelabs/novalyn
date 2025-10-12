use novalyn_core::config::LoadOptions;
use novalyn_core::git::RawCommit;
use novalyn_core::parse::parse_and_classify;

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
fn filters_chore_deps_variants() {
    let td = tempfile::tempdir().unwrap();
    let cfg = novalyn_core::config::load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();
    let commits = vec![
        mk("chore(deps): bump"),
        mk("chore(deps-dev): bump dev"),
        mk("chore(other): keep"),
    ];
    let parsed = parse_and_classify(commits.into(), &cfg);
    assert_eq!(parsed.len(), 1, "only chore(other) should remain");
    assert_eq!(parsed[0].raw.summary, "chore(other): keep");
}

#[test]
fn keeps_breaking_chore_deps() {
    let td = tempfile::tempdir().unwrap();
    let cfg = novalyn_core::config::load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();
    let commits = vec![mk("chore(deps)!: major bump")];
    let parsed = parse_and_classify(commits.into(), &cfg);
    assert_eq!(parsed.len(), 1);
    assert!(parsed[0].breaking);
}
