use assert_fs::TempDir;
use novalyn_core::config::{LoadOptions, load_config};
use novalyn_core::git::RawCommit;
use novalyn_core::parse::{ParsedCommit, parse_and_classify};

fn mk_commit(summary: &str, body: &str) -> RawCommit {
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

#[test]
fn parse_basic() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();
    let commits = vec![
        mk_commit(
            "feat(parser): add new parser",
            "Body\n\nBREAKING CHANGE: format changed",
        ),
        mk_commit("fix: bug fix", ""),
        mk_commit("chore(deps): update deps", ""),
        mk_commit("refactor!: big refactor", "details"),
        mk_commit(
            "style: tabs",
            "Body part\n\nCo-authored-by: Someone <s@e.com>",
        ),
    ];
    let parsed = parse_and_classify(commits.into(), &cfg); // chore(deps) should be filtered
    assert!(parsed.iter().any(|c| c.r#type == "feat" && c.breaking));
    assert!(!parsed.iter().any(|c| c.summary().starts_with("chore")));
    assert!(parsed.iter().any(|c| c.r#type == "refactor" && c.breaking));
    assert!(parsed.iter().any(|c| c.co_authors.len() == 1));
}

trait Summ {
    fn summary(&self) -> &str;
}
impl Summ for ParsedCommit {
    fn summary(&self) -> &str {
        &self.raw.summary
    }
}
