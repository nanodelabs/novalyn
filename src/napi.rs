//! NAPI-RS bindings for Node.js/npm integration
//!
//! This module provides JavaScript-compatible bindings for changelogen functionality,
//! enabling npm package distribution via NAPI-RS.

#![cfg(feature = "napi")]
#![allow(unsafe_code)] // NAPI-RS requires unsafe code for FFI bindings

use crate::authors::{AuthorOptions, Authors};
use crate::changelog;
use crate::config::{self, LoadOptions};
use crate::git;
use crate::parse;
use crate::pipeline::{ExitCode, ReleaseOptions as RustReleaseOptions, run_release};
use crate::render::{render_release_block, RenderContext};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::PathBuf;

/// Options for generating changelog
#[napi(object)]
#[derive(Debug, Clone)]
pub struct GenerateOptions {
    /// Working directory (defaults to current directory)
    pub cwd: Option<String>,
    /// Starting git reference (tag, branch, or commit)
    pub from: Option<String>,
    /// Ending git reference (defaults to HEAD)
    pub to: Option<String>,
    /// Path to output file (defaults to CHANGELOG.md)
    pub output: Option<String>,
    /// Whether to write to file or return as string
    pub write: Option<bool>,
    /// Exclude authors from changelog
    pub exclude_authors: Option<Vec<String>>,
    /// Don't include authors section
    pub no_authors: Option<bool>,
    /// Dry run mode (no file system changes)
    pub dry_run: Option<bool>,
    /// Hide author emails
    pub hide_author_email: Option<bool>,
}

/// Options for release command
#[napi(object)]
#[derive(Debug, Clone)]
pub struct ReleaseOptions {
    /// Working directory (defaults to current directory)
    pub cwd: Option<String>,
    /// Starting git reference (tag, branch, or commit)
    pub from: Option<String>,
    /// Ending git reference (defaults to HEAD)
    pub to: Option<String>,
    /// Explicit version to use (overrides inference)
    pub new_version: Option<String>,
    /// Path to output file (defaults to CHANGELOG.md)
    pub output: Option<String>,
    /// Sign git tag
    pub sign: Option<bool>,
    /// Exclude authors from changelog
    pub exclude_authors: Option<Vec<String>>,
    /// Don't include authors section
    pub no_authors: Option<bool>,
    /// Dry run mode (no file system changes)
    pub dry_run: Option<bool>,
    /// Skip confirmation prompts
    pub yes: Option<bool>,
    /// Hide author emails
    pub hide_author_email: Option<bool>,
    /// Clean working tree required
    pub clean: Option<bool>,
}

/// Result of generate operation
#[napi(object)]
#[derive(Debug, Clone)]
pub struct GenerateResult {
    /// Generated changelog markdown
    pub content: String,
    /// Number of commits processed
    pub commits: u32,
    /// Version tag used
    pub version: Option<String>,
}

/// Result of release operation
#[napi(object)]
#[derive(Debug, Clone)]
pub struct ReleaseResult {
    /// Generated changelog markdown
    pub content: String,
    /// Number of commits processed
    pub commits: u32,
    /// Previous version
    pub previous_version: String,
    /// New version created
    pub new_version: String,
    /// Whether a tag was created
    pub tag_created: bool,
}

/// Generate changelog from git history
///
/// # Arguments
/// * `options` - Configuration options for changelog generation
///
/// # Returns
/// Promise resolving to generated changelog content and metadata
///
/// # Example
/// ```javascript
/// const { generate } = require('changelogen');
///
/// const result = await generate({
///   from: 'v1.0.0',
///   to: 'HEAD',
///   write: true
/// });
///
/// console.log(`Generated changelog with ${result.commits} commits`);
/// ```
#[napi]
pub async fn generate(options: Option<GenerateOptions>) -> Result<GenerateResult> {
    let opts = options.unwrap_or_else(|| GenerateOptions {
        cwd: None,
        from: None,
        to: None,
        output: None,
        write: None,
        exclude_authors: None,
        no_authors: None,
        dry_run: None,
        hide_author_email: None,
    });

    // Get working directory
    let cwd = if let Some(ref path) = opts.cwd {
        PathBuf::from(path)
    } else {
        std::env::current_dir()
            .map_err(|e| Error::from_reason(format!("Failed to get current directory: {}", e)))?
    };

    // Run in blocking task since git operations are synchronous
    let result = tokio::task::spawn_blocking(move || {
        // Load configuration
        let cfg = config::load_config(LoadOptions {
            cwd: &cwd,
            cli_overrides: None,
        })
        .map_err(|e| format!("Failed to load config: {}", e))?;

        // Detect git repo
        let repo = git::detect_repo(&cwd).map_err(|e| format!("Failed to detect repo: {}", e))?;

        let to_ref = opts.to.clone().unwrap_or_else(|| "HEAD".into());
        let from_ref = opts.from.clone();

        // Collect commits
        let raw = git::commits_between(&repo, from_ref.as_deref(), &to_ref)
            .map_err(|e| format!("Failed to collect commits: {}", e))?;

        // Parse & classify
        let parsed = parse::parse_and_classify(raw, &cfg);

        // Determine version
        let prev_tag = git::last_tag(&repo).ok().flatten();
        let previous_version = prev_tag
            .as_ref()
            .and_then(|t| semver::Version::parse(t.trim_start_matches('v')).ok())
            .unwrap_or_else(|| semver::Version::new(0, 0, 0));

        let (next_version, _) = parse::infer_version(&previous_version, &parsed, None);

        // Collect authors
        let no_authors = opts.no_authors.unwrap_or(false);
        let authors = if no_authors {
            None
        } else {
            Some(Authors::collect(
                &parsed,
                &AuthorOptions {
                    exclude: opts.exclude_authors.clone().unwrap_or_default(),
                    hide_author_email: opts.hide_author_email.unwrap_or(false),
                    no_authors,
                },
            ))
        };

        // Render changelog block
        let rc = RenderContext {
            version: &next_version,
            previous_version: Some(&previous_version),
            commits: &parsed,
            authors: authors.as_ref(),
            repo: cfg.repo.as_ref(),
            cfg: &cfg,
            previous_tag: prev_tag.as_deref(),
            current_ref: &to_ref,
        };
        let block = render_release_block(&rc);

        // Optionally write to file
        if opts.write.unwrap_or(false) && !opts.dry_run.unwrap_or(false) {
            changelog::write_or_update_changelog(&cwd, &block)
                .map_err(|e| format!("Failed to write changelog: {}", e))?;
        }

        Ok::<_, String>((
            block,
            parsed.len() as u32,
            Some(format!("v{}", next_version)),
        ))
    })
    .await
    .map_err(|e| Error::from_reason(format!("Task panicked: {}", e)))?
    .map_err(|e| Error::from_reason(e))?;

    Ok(GenerateResult {
        content: result.0,
        commits: result.1,
        version: result.2,
    })
}

/// Perform full release: version bump, changelog generation, and git tag creation
///
/// # Arguments
/// * `options` - Configuration options for release
///
/// # Returns
/// Promise resolving to release result with version information
///
/// # Example
/// ```javascript
/// const { release } = require('changelogen');
///
/// const result = await release({
///   dryRun: true,
///   yes: true
/// });
///
/// console.log(`Release ${result.newVersion} (from ${result.previousVersion})`);
/// ```
#[napi]
pub async fn release(options: Option<ReleaseOptions>) -> Result<ReleaseResult> {
    let opts = options.unwrap_or_else(|| ReleaseOptions {
        cwd: None,
        from: None,
        to: None,
        new_version: None,
        output: None,
        sign: None,
        exclude_authors: None,
        no_authors: None,
        dry_run: None,
        yes: None,
        hide_author_email: None,
        clean: None,
    });

    // Get working directory
    let cwd = if let Some(ref path) = opts.cwd {
        PathBuf::from(path)
    } else {
        std::env::current_dir()
            .map_err(|e| Error::from_reason(format!("Failed to get current directory: {}", e)))?
    };

    // Run in blocking task
    let result = tokio::task::spawn_blocking(move || {
        let new_version = opts
            .new_version
            .as_ref()
            .and_then(|v| semver::Version::parse(v.trim_start_matches('v')).ok());

        let release_opts = RustReleaseOptions {
            cwd: cwd.clone(),
            from: opts.from,
            to: opts.to,
            dry_run: opts.dry_run.unwrap_or(false),
            new_version,
            no_authors: opts.no_authors.unwrap_or(false),
            exclude_authors: opts.exclude_authors.unwrap_or_default(),
            hide_author_email: opts.hide_author_email.unwrap_or(false),
            clean: opts.clean.unwrap_or(false),
            sign: opts.sign.unwrap_or(false),
            yes: opts.yes.unwrap_or(false),
        };

        run_release(release_opts).map_err(|e| format!("Release failed: {}", e))
    })
    .await
    .map_err(|e| Error::from_reason(format!("Task panicked: {}", e)))?
    .map_err(|e| Error::from_reason(e))?;

    // Read the generated changelog content
    let content = std::fs::read_to_string(&result.changelog_path)
        .map_err(|e| Error::from_reason(format!("Failed to read changelog: {}", e)))?;

    Ok(ReleaseResult {
        content,
        commits: result.commit_count as u32,
        previous_version: result
            .previous
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "0.0.0".to_string()),
        new_version: result.version.to_string(),
        tag_created: result.wrote && result.exit == ExitCode::Success,
    })
}

/// Get the next semantic version based on commits
///
/// # Arguments
/// * `options` - Configuration options
///
/// # Returns
/// Promise resolving to the next version string
///
/// # Example
/// ```javascript
/// const { showVersion } = require('changelogen');
///
/// const version = await showVersion({ from: 'v1.0.0' });
/// console.log(`Next version: ${version}`);
/// ```
#[napi]
pub async fn show_version(options: Option<GenerateOptions>) -> Result<String> {
    let opts = options.unwrap_or_else(|| GenerateOptions {
        cwd: None,
        from: None,
        to: None,
        output: None,
        write: None,
        exclude_authors: None,
        no_authors: None,
        dry_run: None,
        hide_author_email: None,
    });

    // Get working directory
    let cwd = if let Some(ref path) = opts.cwd {
        PathBuf::from(path)
    } else {
        std::env::current_dir()
            .map_err(|e| Error::from_reason(format!("Failed to get current directory: {}", e)))?
    };

    // Run in blocking task
    let version = tokio::task::spawn_blocking(move || {
        // Load configuration
        let cfg = config::load_config(LoadOptions {
            cwd: &cwd,
            cli_overrides: None,
        })
        .map_err(|e| format!("Failed to load config: {}", e))?;

        // Detect git repo
        let repo = git::detect_repo(&cwd).map_err(|e| format!("Failed to detect repo: {}", e))?;

        let to_ref = opts.to.clone().unwrap_or_else(|| "HEAD".into());
        let from_ref = opts.from.clone();

        // Collect commits
        let raw = git::commits_between(&repo, from_ref.as_deref(), &to_ref)
            .map_err(|e| format!("Failed to collect commits: {}", e))?;

        // Parse & classify
        let parsed = parse::parse_and_classify(raw, &cfg);

        // Determine version
        let prev_tag = git::last_tag(&repo).ok().flatten();
        let previous_version = prev_tag
            .as_ref()
            .and_then(|t| semver::Version::parse(t.trim_start_matches('v')).ok())
            .unwrap_or_else(|| semver::Version::new(0, 0, 0));

        let (next_version, _) = parse::infer_version(&previous_version, &parsed, None);

        Ok::<_, String>(format!("v{}", next_version))
    })
    .await
    .map_err(|e| Error::from_reason(format!("Task panicked: {}", e)))?
    .map_err(|e| Error::from_reason(e))?;

    Ok(version)
}
