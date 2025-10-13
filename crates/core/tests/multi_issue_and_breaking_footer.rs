use novalyn_core::config::LoadOptions;
use novalyn_core::git::RawCommit;
use novalyn_core::parse::parse_and_classify;

/// Create a RawCommit with the given summary and body for multi-issue and breaking footer tests.
fn mk(summary: &str, body: &str) -> RawCommit {
    RawCommit {
        id: "x".into(),
        short_id: "x".into(),
        summary: summary.into(),
        body: body.into(),
        author_name: "A".into(),
        author_email: "a@b.c".into(),
        timestamp: 0,
    }
}

/// Test extraction of multiple issues and breaking change footers from commit messages.
#[test]
fn extracts_multiple_issues_grouped_and_body() {
    let td = tempfile::tempdir().unwrap();
    let cfg = novalyn_core::config::load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();
    let c = mk(
        "fix(parser): handle edge (#12, #34)",
        "Implements fix refs #56 #78\n\nFooter: note\nBREAKING CHANGE: behaviour changed significantly\n    Additional explanation line\nAnother: value",
    );
    let parsed = parse_and_classify(vec![c].into(), &cfg);
    assert_eq!(parsed[0].issues, vec![12, 34, 56, 78]);
    assert!(parsed[0].breaking, "breaking change detected via footer");
    // Multi-line not yet captured; just ensure footer exists for now
    let breaking_footer = parsed[0]
        .footers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("BREAKING CHANGE"))
        .unwrap();
    assert!(breaking_footer.1.contains("behaviour changed"));
    assert!(
        breaking_footer.1.contains("Additional explanation line"),
        "expects continuation line captured"
    );
}
