use crate::{
    authors::Authors,
    config::ResolvedConfig,
    parse::ParsedCommit,
    repository::{Repository, format_compare_changes},
};

#[derive(Debug)]
pub struct RenderContext<'a> {
    pub version: &'a semver::Version,
    pub previous_version: Option<&'a semver::Version>,
    pub commits: &'a [ParsedCommit],
    pub authors: Option<&'a Authors>,
    pub repo: Option<&'a Repository>,
    pub cfg: &'a ResolvedConfig,
    pub previous_tag: Option<&'a str>,
    pub current_ref: &'a str,
}

pub fn render_release_block(ctx: &RenderContext<'_>) -> String {
    let mut out = String::new();
    // Header
    out.push_str(&format!("## v{}", ctx.version));
    out.push('\n');
    if let (Some(_prev), Some(repo), Some(prev_tag)) =
        (ctx.previous_version, ctx.repo, ctx.previous_tag)
    {
        if let Some(compare) =
            format_compare_changes(None, prev_tag, &format!("v{}", ctx.version), Some(repo))
        {
            out.push_str(&compare);
            out.push('\n');
        }
    }
    // Group commits by type order
    for tc in &ctx.cfg.types {
        if !tc.enabled {
            continue;
        }
        let mut section_lines: Vec<String> = Vec::new();
        let mut candidates: Vec<&ParsedCommit> =
            ctx.commits.iter().filter(|c| c.r#type == tc.key).collect();
        // Already chronological by pipeline; ensure stable tie-break by original index (defensive)
        candidates.sort_by_key(|c| c.index);
        for c in candidates {
            let mut line = String::new();
            if let Some(scope) = &c.scope {
                line.push_str(&format!("* {}({}): {}", tc.emoji, scope, c.description));
            } else {
                line.push_str(&format!("* {}: {}", tc.emoji, c.description));
            }
            if c.breaking {
                line.push_str(" (BREAKING)");
            }
            if !c.issues.is_empty() {
                let refs: Vec<String> = if let Some(repo) = ctx.repo {
                    c.issues
                        .iter()
                        .map(|n| format!("[#{}]({})", n, repo.issue_url(*n)))
                        .collect()
                } else {
                    c.issues.iter().map(|n| format!("#{}", n)).collect()
                };
                line.push_str(&format!(" ({})", refs.join(", ")));
            }
            section_lines.push(line);
        }
        if !section_lines.is_empty() {
            out.push('\n');
            out.push_str(&format!("### {} {}", tc.emoji, tc.title));
            out.push('\n');
            for l in section_lines {
                out.push_str(&l);
                out.push('\n');
            }
        }
    }
    // Contributors
    if let Some(auths) = ctx.authors {
        if !auths.suppressed && !auths.list.is_empty() {
            out.push('\n');
            out.push_str("### Contributors\n");
            for a in &auths.list {
                if let Some(email) = &a.email {
                    out.push_str(&format!("- {} <{}>\n", a.name, email));
                } else {
                    out.push_str(&format!("- {}\n", a.name));
                }
            }
        }
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{ResolvedConfig, default_types},
        git::RawCommit,
        parse::ParsedCommit,
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

    fn mk_commit(t: &str, desc: &str) -> ParsedCommit {
        ParsedCommit {
            raw: RawCommit {
                id: "1".into(),
                short_id: "1".into(),
                summary: format!("{}: {}", t, desc),
                body: String::new(),
                author_name: "A".into(),
                author_email: "a@x".into(),
                timestamp: 0,
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
            index: 0,
        }
    }

    #[test]
    fn basic_render() {
        let cfg = dummy_cfg();
        let commits = vec![mk_commit("feat", "add"), mk_commit("fix", "bug")];
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
        assert!(txt.contains("## v1.0.0"));
        assert!(txt.contains("### ✨ Features"));
        assert!(txt.contains("### 🐞 Bug Fixes"));
    }
}
