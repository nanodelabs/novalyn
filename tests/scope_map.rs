use changelogen::config::{LoadOptions, RawConfig};
use changelogen::git::RawCommit;
use changelogen::parse::parse_and_classify;

fn mk(summary: &str) -> RawCommit {
    RawCommit {
        id: "x".into(),
        short_id: "x".into(),
        summary: summary.into(),
        body: String::new(),
        author_name: "A".into(),
        author_email: "a@b.c".into(),
        timestamp: 0,
    }
}

#[test]
fn scope_mapping_replacement_and_removal() {
    let td = tempfile::tempdir().unwrap();
    let cli = RawConfig {
        scope_map: Some({
            let mut m = std::collections::BTreeMap::new();
            m.insert("core".into(), "runtime".into()); // replacement
            m.insert("temp".into(), String::new()); // removal
            m
        }),
        ..Default::default()
    };
    let cfg = changelogen::config::load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: Some(cli),
    })
    .unwrap();
    let commits = vec![
        mk("feat(core): change"),
        mk("fix(temp): fix"),
        mk("docs(other): doc"),
    ];
    let parsed = parse_and_classify(commits, &cfg);
    let mut core_mapped = false;
    let mut temp_removed = false;
    for c in parsed {
        if c.description.contains("change") {
            assert_eq!(c.scope.as_deref(), Some("runtime"));
            core_mapped = true;
        }
        if c.description.contains("fix") {
            assert!(c.scope.is_none());
            temp_removed = true;
        }
    }
    assert!(core_mapped && temp_removed);
}
