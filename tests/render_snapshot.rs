use changelogen::{
    config::{ResolvedConfig, default_types},
    git::RawCommit,
    parse::ParsedCommit,
    render::{RenderContext, render_release_block},
};

fn dummy_cfg() -> ResolvedConfig {
    ResolvedConfig {
        scope_map: Default::default(),
        types: default_types(),
        new_version: None,
        warnings: vec![],
        github_token: None,
        cwd: std::path::PathBuf::from("."),
        source_file: None,
        repo: None,
    }
}

fn mk(idx: usize, t: &str, desc: &str) -> ParsedCommit {
    ParsedCommit {
        raw: RawCommit {
            id: format!("{idx}"),
            short_id: format!("{idx}"),
            summary: format!("{t}: {desc}"),
            body: String::new(),
            author_name: "A".into(),
            author_email: "a@x".into(),
            timestamp: idx as i64,
        },
        r#type: t.into(),
        scope: None,
        description: desc.into(),
        body: String::new(),
        footers: vec![],
        breaking: false,
        issues: vec![],
        co_authors: vec![],
        type_cfg: None,
        index: idx,
    }
}

#[test]
fn deterministic_ordering() {
    let cfg = dummy_cfg();
    let mut commits = vec![
        mk(2, "feat", "third"),
        mk(0, "feat", "first"),
        mk(1, "feat", "second"),
    ];
    // shuffle purposely out of order; renderer should sort by index
    let rc = RenderContext {
        version: &semver::Version::parse("1.0.0").unwrap(),
        previous_version: None,
        commits: &commits,
        authors: None,
        repo: None,
        cfg: &cfg,
        previous_tag: None,
        current_ref: "HEAD",
    };
    let txt = render_release_block(&rc);
    let feat_section = txt.split("### ✨ Features").nth(1).unwrap();
    let lines: Vec<&str> = feat_section
        .lines()
        .filter(|l| l.starts_with("* "))
        .collect();
    assert_eq!(lines, vec!["* ✨: first", "* ✨: second", "* ✨: third"]);
    // re-order commits vector differently, still expect same ordering
    commits.swap(0, 1);
    let rc2 = RenderContext {
        version: &semver::Version::parse("1.0.0").unwrap(),
        previous_version: None,
        commits: &commits,
        authors: None,
        repo: None,
        cfg: &cfg,
        previous_tag: None,
        current_ref: "HEAD",
    };
    let txt2 = render_release_block(&rc2);
    let feat_section2 = txt2.split("### ✨ Features").nth(1).unwrap();
    let lines2: Vec<&str> = feat_section2
        .lines()
        .filter(|l| l.starts_with("* "))
        .collect();
    assert_eq!(lines2, vec!["* ✨: first", "* ✨: second", "* ✨: third"]);
}
