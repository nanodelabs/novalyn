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
fn feat_bang_breaking() {
    let td = tempfile::tempdir().unwrap();
    let cfg = novalyn_core::config::load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();
    let commits = parse_and_classify(
        vec![mk("feat!: change api"), mk("feat(core)!: change core")].into(),
        &cfg,
    );
    assert_eq!(commits.len(), 2);
    assert!(commits.iter().all(|c| c.breaking));
    assert!(commits.iter().any(|c| c.scope.as_deref() == Some("core")));
}
