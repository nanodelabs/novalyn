use novalyn_core::{
    config::{ResolvedConfig, default_types},
    git::RawCommit,
    parse::ParsedCommit,
    render::{RenderContext, render_release_block},
};

fn cfg() -> ResolvedConfig {
    ResolvedConfig {
        scope_map: Default::default(),
        types: default_types(),
        new_version: None,
        warnings: vec![].into(),
        github_token: None,
        cwd: ".".into(),
        source_file: None,
        repo: None,
    }
}

fn mk(idx: usize, t: &str, desc: &str) -> ParsedCommit {
    ParsedCommit {
        raw: RawCommit {
            id: format!("{idx}").into(),
            short_id: format!("{idx}").into(),
            summary: format!("{t}: {desc}").into(),
            body: String::new().into(),
            author_name: "A".into(),
            author_email: "a@x".into(),
            timestamp: idx as i64,
        },
        r#type: t.into(),
        scope: None,
        description: desc.into(),
        body: String::new().into(),
        footers: vec![].into(),
        breaking: false,
        issues: vec![].into(),
        co_authors: vec![].into(),
        type_cfg: None,
        index: idx,
    }
}

#[test]
fn render_block_snapshot() {
    let cfg = cfg();
    let commits = vec![mk(0, "feat", "add A"), mk(1, "fix", "bug B")];
    let rc = RenderContext {
        version: &semver::Version::parse("1.2.0").unwrap(),
        previous_version: Some(&semver::Version::parse("1.1.0").unwrap()),
        commits: &commits,
        authors: None,
        repo: None,
        cfg: &cfg,
        previous_tag: Some("v1.1.0"),
        current_ref: "HEAD",
    };
    let txt = render_release_block(&rc);
    insta::assert_snapshot!("render_block", txt);
}
