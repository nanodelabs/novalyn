use crate::{
    authors::Authors,
    config::ResolvedConfig,
    parse::ParsedCommit,
    repository::{Repository, format_compare_changes},
};
use ecow::EcoString;

/// Context for rendering a changelog release block.
///
/// Contains all data needed to generate formatted markdown output.
#[derive(Debug)]
pub struct RenderContext<'a> {
    pub version: &'a semver::Version,
    pub previous_version: Option<&'a semver::Version>,
    pub commits: &'a [ParsedCommit],
    pub authors: Option<&'a Authors>,
    pub repo: Option<&'a Repository>,
    pub cfg: &'a ResolvedConfig,
    pub previous_tag: Option<&'a str>,
    /// Current git reference (branch or tag name)
    pub current_ref: &'a str,
}

/// Render a changelog release block in markdown format with parallel section rendering.
///
/// Generates a formatted release section with:
/// - Version header with compare link
/// - Commits grouped by type (features, fixes, etc.)
/// - Breaking change indicators
/// - Issue references with links
/// - Contributors section
///
/// # Arguments
/// * `ctx` - Render context with commits, version, and configuration
///
/// # Returns
/// Formatted markdown release block as a string
pub fn render_release_block(ctx: &RenderContext<'_>) -> EcoString {
    use rayon::prelude::*;

    let mut out = String::new();
    // Header
    out.push_str(&format!("## v{}", ctx.version));
    out.push('\n');
    if let (Some(_prev), Some(repo), Some(prev_tag)) =
        (ctx.previous_version, ctx.repo, ctx.previous_tag)
        && let Some(compare) =
            format_compare_changes(None, prev_tag, &format!("v{}", ctx.version), Some(repo))
    {
        out.push_str(&compare);
        out.push('\n');
    }

    // Render sections in parallel for better performance
    let sections: Vec<(usize, String)> = ctx
        .cfg
        .types
        .par_iter()
        .enumerate()
        .filter(|(_, tc)| tc.enabled)
        .filter_map(|(idx, tc)| {
            let mut candidates: Vec<&ParsedCommit> =
                ctx.commits.iter().filter(|c| c.r#type == tc.key).collect();

            if candidates.is_empty() {
                return None;
            }

            // Already chronological by pipeline; ensure stable tie-break by original index
            candidates.sort_by_key(|c| c.index);

            let mut section = String::new();
            section.push('\n');
            section.push_str(&format!("### {} {}", tc.emoji, tc.title));
            section.push('\n');

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
                section.push_str(&line);
                section.push('\n');
            }

            Some((idx, section))
        })
        .collect();

    // Append sections in original order to maintain deterministic output
    for (_, section) in sections {
        out.push_str(&section);
    }

    // Contributors
    if let Some(auths) = ctx.authors
        && !auths.suppressed
        && !auths.list.is_empty()
    {
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
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.into()
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
            warnings: vec![].into(),
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
                summary: format!("{}: {}", t, desc).into(),
                body: EcoString::new(),
                author_name: "A".into(),
                author_email: "a@x".into(),
                timestamp: 0,
            },
            r#type: t.into(),
            scope: None,
            description: desc.into(),
            body: EcoString::new(),
            footers: vec![].into(),
            breaking: false,
            issues: vec![].into(),
            co_authors: vec![].into(),
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
