use crate::{
    authors::{AuthorOptions, Authors},
    changelog,
    config::{self, LoadOptions},
    git, parse,
    render::{RenderContext, render_release_block},
};

use anyhow::Result;
use demand::Confirm;
use ecow::{EcoString, EcoVec};
use tracing::{debug, info, instrument, warn};

/// Interactive confirmation prompt for release operations.
///
/// Uses the `demand` crate to display a confirmation dialog in the terminal
/// unless `yes_flag` is true, in which case it auto-confirms without user interaction.
///
/// # Arguments
/// * `message` - Prompt message to display/log
/// * `yes_flag` - If true, skip interactive prompt and auto-confirm
///
/// # Returns
/// `Ok(true)` if confirmed, `Ok(false)` if declined or cancelled, `Err` on prompt error
fn confirm_action(message: &str, yes_flag: bool) -> Result<bool> {
    if yes_flag {
        tracing::debug!("Auto-confirming: {}", message);
        return Ok(true);
    }

    let confirm = Confirm::new(message).affirmative("Yes").negative("No");
    match confirm.run() {
        Ok(choice) => Ok(choice),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::Interrupted {
                tracing::info!("Prompt cancelled by user");
                Ok(false)
            } else {
                Err(e.into())
            }
        }
    }
}

/// Exit codes returned by release pipeline.
///
/// Following standard Unix conventions for process exit codes.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Successful release operation
    Success = 0,
    /// No changes detected (idempotent run)
    NoChange = 3,
}

/// Configuration options for release pipeline execution.
///
/// Controls all aspects of changelog generation, version bumping,
/// and git operations.
pub struct ReleaseOptions {
    pub cwd: std::path::PathBuf,
    pub from: Option<EcoString>,
    pub to: Option<EcoString>, // default HEAD
    pub dry_run: bool,
    pub new_version: Option<semver::Version>,
    pub no_authors: bool,
    pub exclude_authors: EcoVec<EcoString>,
    pub hide_author_email: bool,
    pub clean: bool,
    pub sign: bool,
    pub yes: bool,
    /// Whether to resolve author emails to GitHub handles
    pub github_alias: bool,
    /// GitHub API token for handle resolution
    pub github_token: Option<EcoString>,
}

/// Result of a release pipeline execution.
///
/// Contains information about the generated release including version,
/// file paths, and operation status.
pub struct ReleaseOutcome {
    pub version: semver::Version,
    pub previous: Option<semver::Version>,
    pub wrote: bool,
    pub changelog_path: std::path::PathBuf,
    pub commit_count: usize,
    /// Process exit code
    pub exit: ExitCode,
}

/// Execute the complete release pipeline asynchronously.
///
/// Orchestrates all steps of changelog generation:
/// 1. Load configuration
/// 2. Detect git repository and commits
/// 3. Parse and classify conventional commits
/// 4. Infer semantic version
/// 5. Collect and resolve authors
/// 6. Render changelog block
/// 7. Write to CHANGELOG.md
/// 8. Update Cargo.toml version
/// 9. Create git commit and tag
///
/// # Arguments
/// * `opts` - Release configuration options
///
/// # Returns
/// * `Ok(ReleaseOutcome)` - Successful release with details
/// * `Err` - Pipeline error occurred
///
/// # Errors
/// Returns error if configuration loading, git operations, or file writes fail
#[instrument(skip_all, fields(cwd = %opts.cwd.display()))]
pub async fn run_release_async(opts: ReleaseOptions) -> Result<ReleaseOutcome> {
    // 1. Load config (inject CLI overrides for new_version & author flags in future)
    let cfg = config::load_config_async(LoadOptions {
        cwd: &opts.cwd,
        cli_overrides: None,
    })
    .await?;
    debug!(types = cfg.types.len(), "config_loaded");

    // 2. Detect git repo & current ref
    let mut repo = git::detect_repo(&opts.cwd)?;
    if opts.clean && git::is_dirty(&repo)? {
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
        .unwrap_or_else(|| semver::Version::new(0, 0, 0));
    let (next_version, _bump) = {
        let _span = tracing::span!(tracing::Level::DEBUG, "infer_version").entered();
        parse::infer_version(&previous_version, &parsed, opts.new_version.clone())
    };
    info!(version = %next_version, "version_inferred");

    // 7. Authors
    let authors = if opts.no_authors {
        None
    } else {
        use ecow::{EcoString, EcoVec};

        let aliases = scc::HashMap::with_hasher(foldhash::quality::RandomState::default());

        let exclude: EcoVec<EcoString> = opts.exclude_authors.clone();

        let mut authors = Authors::collect(
            &parsed,
            &AuthorOptions {
                exclude,
                hide_author_email: opts.hide_author_email,
                no_authors: opts.no_authors,
                aliases,
                github_token: opts.github_token.as_ref().map(|s| s.to_string()),
                enable_github_aliasing: opts.github_alias,
            },
        );

        // If GitHub aliasing is enabled and we have a token, resolve handles
        if opts.github_alias {
            if let Some(ref token) = opts.github_token {
                // Now we're already in async context, so we can just await
                if let Err(e) = authors.resolve_github_handles(token).await {
                    warn!("failed to resolve GitHub handles: {}", e);
                }
            } else {
                debug!(
                    "GitHub aliasing enabled but no token provided; skipping handle resolution (set GITHUB_TOKEN or GH_TOKEN env var, or use --no-github-alias to disable)"
                );
            }
        }

        Some(authors)
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
        // Confirm changelog update unless --yes was specified
        let should_write = confirm_action("Update CHANGELOG.md?", opts.yes)?;

        if should_write {
            let _span = tracing::span!(tracing::Level::DEBUG, "write_changelog").entered();
            changelog::write_or_update_changelog_async(&opts.cwd, &block).await?
        } else {
            false
        }
    };
    if changed && !opts.dry_run {
        // Confirm tag creation unless --yes was specified
        let should_tag = confirm_action(&format!("Create git tag v{}?", next_version), opts.yes)?;

        if should_tag {
            // create tag (annotated optionally sign placeholder)
            let tag_name = format!("v{}", next_version);
            let tag_msg = format!("v{}", next_version);
            let _ = {
                let _span = tracing::span!(tracing::Level::DEBUG, "tag").entered();
                git::create_tag(&mut repo, &tag_name, &tag_msg, true)
            };
        }
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

/// Execute the complete release pipeline synchronously (for backward compatibility).
///
/// This is a wrapper around `run_release_async` that blocks on the async runtime.
/// Consider using `run_release_async` directly if you're already in an async context.
///
/// # Arguments
/// * `opts` - Release configuration options
///
/// # Returns
/// * `Ok(ReleaseOutcome)` - Successful release with details
/// * `Err` - Pipeline error occurred
#[instrument(skip_all, fields(cwd = %opts.cwd.display()))]
pub fn run_release(opts: ReleaseOptions) -> Result<ReleaseOutcome> {
    // Create a runtime and block on the async version
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_release_async(opts))
}
