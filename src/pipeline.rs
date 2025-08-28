use crate::{
    authors::{AuthorOptions, Authors},
    changelog,
    config::{self, LoadOptions},
    git, parse,
    render::{RenderContext, render_release_block},
};

use anyhow::Result;
use tracing::{debug, info, instrument};

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    NoChange = 3,
}

pub struct ReleaseOptions {
    pub cwd: std::path::PathBuf,
    pub from: Option<String>,
    pub to: Option<String>, // default HEAD
    pub dry_run: bool,
    pub new_version: Option<semver::Version>,
    pub no_authors: bool,
    pub exclude_authors: Vec<String>,
    pub hide_author_email: bool,
    pub clean: bool,
    pub sign: bool,
}

pub struct ReleaseOutcome {
    pub version: semver::Version,
    pub previous: Option<semver::Version>,
    pub wrote: bool,
    pub changelog_path: std::path::PathBuf,
    pub commit_count: usize,
    pub exit: ExitCode,
}

#[instrument(skip_all, fields(cwd = %opts.cwd.display()))]
pub fn run_release(opts: ReleaseOptions) -> Result<ReleaseOutcome> {
    // 1. Load config (inject CLI overrides for new_version & author flags in future)
    let cfg = config::load_config(LoadOptions {
        cwd: &opts.cwd,
        cli_overrides: None,
    })?;
    debug!(types = cfg.types.len(), "config_loaded");

    // 2. Detect git repo & current ref
    let repo = git::detect_repo(&opts.cwd)?;
    if opts.clean && git::is_dirty(&repo, true)? {
        anyhow::bail!("working tree dirty (use --clean to enforce cleanliness or commit changes)");
    }
    let head = opts.to.clone().unwrap_or_else(|| "HEAD".into());

    // 3. Determine previous tag
    let prev_tag = git::last_tag(&repo)?; // Option<String>

    // 4. Collect commits between prev_tag and head
    let raw = {
        let _span = tracing::span!(tracing::Level::DEBUG, "collect_commits").entered();
        git::commits_between(&repo, prev_tag.as_deref(), &head)?
    };
    debug!(count = raw.len(), "commits_collected");

    // 5. Parse & classify
    let parsed = {
        let _span = tracing::span!(tracing::Level::DEBUG, "parse_classify").entered();
        parse::parse_and_classify(raw, &cfg)
    };
    debug!(count = parsed.len(), "commits_parsed");

    // 6. Version inference: use 0.0.0 if no prev tag
    let previous_version = prev_tag
        .as_ref()
        .and_then(|t| semver::Version::parse(t.trim_start_matches('v')).ok())
        .unwrap_or_else(|| semver::Version::parse("0.0.0").unwrap());
    let (next_version, _bump) = {
        let _span = tracing::span!(tracing::Level::DEBUG, "infer_version").entered();
        parse::infer_version(&previous_version, &parsed, opts.new_version.clone())
    };
    info!(version = %next_version, "version_inferred");

    // 7. Authors
    let authors = if opts.no_authors {
        None
    } else {
        Some(Authors::collect(
            &parsed,
            &AuthorOptions {
                exclude: opts.exclude_authors.clone(),
                hide_author_email: opts.hide_author_email,
                no_authors: opts.no_authors,
            },
        ))
    };

    // 8. Render
    let rc = RenderContext {
        version: &next_version,
        previous_version: Some(&previous_version),
        commits: &parsed,
        authors: authors.as_ref(),
        repo: cfg.repo.as_ref(),
        cfg: &cfg,
        previous_tag: prev_tag.as_deref(),
        current_ref: &head,
    };
    let block = {
        let _span = tracing::span!(tracing::Level::DEBUG, "render").entered();
        render_release_block(&rc)
    };

    // 9. Update changelog & tag
    let changed = if opts.dry_run {
        false
    } else {
        let _span = tracing::span!(tracing::Level::DEBUG, "write_changelog").entered();
        changelog::write_or_update_changelog(&opts.cwd, &block)?
    };
    if changed && !opts.dry_run {
        // create tag (annotated optionally sign placeholder)
        let tag_name = format!("v{}", next_version);
        let tag_msg = format!("v{}", next_version);
        let _ = {
            let _span = tracing::span!(tracing::Level::DEBUG, "tag").entered();
            git::create_tag(&repo, &tag_name, &tag_msg, true)
        };
    }

    let exit = if changed {
        ExitCode::Success
    } else {
        ExitCode::NoChange
    };
    Ok(ReleaseOutcome {
        version: next_version.clone(),
        previous: Some(previous_version.clone()),
        wrote: changed,
        changelog_path: opts.cwd.join("CHANGELOG.md"),
        commit_count: rc.commits.len(),
        exit,
    })
}
