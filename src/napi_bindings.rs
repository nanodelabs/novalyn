//! NAPI bindings for npm package distribution
//!
//! This module provides JavaScript-compatible bindings for core novalyn functionality
//! using NAPI-RS. It enables novalyn to be published and consumed as an npm package
//! while maintaining the performance benefits of Rust.

use crate::pipeline::{run_release, ReleaseOptions};
use ecow::{eco_vec, EcoString};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::path::PathBuf;

/// JavaScript-compatible configuration options
#[napi(object)]
pub struct JsConfigOptions {
    /// Working directory (defaults to current directory)
    pub cwd: Option<String>,
    /// Starting reference (commit/tag/branch)
    pub from: Option<String>,
    /// Ending reference (commit/tag/branch)
    pub to: Option<String>,
    /// Override the new version
    pub new_version: Option<String>,
    /// Exclude authors by name or email
    pub exclude_authors: Option<Vec<String>>,
    /// Hide author emails in output
    pub hide_author_email: Option<bool>,
    /// Suppress all author attribution
    pub no_authors: Option<bool>,
    /// GitHub token for API access
    pub github_token: Option<String>,
    /// Disable GitHub handle aliasing
    pub no_github_alias: Option<bool>,
    /// Dry run mode (don't write files)
    pub dry_run: Option<bool>,
}

impl From<JsConfigOptions> for ReleaseOptions {
    fn from(js_opts: JsConfigOptions) -> Self {
        let cwd = js_opts
            .cwd
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        ReleaseOptions {
            cwd,
            from: js_opts.from.map(EcoString::from),
            to: js_opts.to.map(EcoString::from),
            dry_run: js_opts.dry_run.unwrap_or(false),
            new_version: js_opts
                .new_version
                .and_then(|v| semver::Version::parse(&v).ok()),
            no_authors: js_opts.no_authors.unwrap_or(false),
            exclude_authors: js_opts
                .exclude_authors
                .map(|v| v.into_iter().map(EcoString::from).collect())
                .unwrap_or_else(|| eco_vec![]),
            hide_author_email: js_opts.hide_author_email.unwrap_or(false),
            clean: false,
            sign: false,
            yes: true, // Auto-confirm for npm API usage
            github_alias: !js_opts.no_github_alias.unwrap_or(false),
            github_token: js_opts.github_token.map(EcoString::from),
        }
    }
}

/// Result of version inference
#[napi(object)]
pub struct JsVersionResult {
    /// The inferred version
    pub version: String,
    /// Type of version bump (major, minor, patch, or none)
    pub bump_type: String,
}

/// Result of changelog generation
#[napi(object)]
pub struct JsGenerateResult {
    /// The generated markdown content
    pub markdown: String,
    /// The version
    pub version: String,
    /// Number of commits processed
    pub commit_count: u32,
}

/// Result of a full release operation
#[napi(object)]
pub struct JsReleaseResult {
    /// The version released
    pub version: String,
    /// Number of commits processed
    pub commit_count: u32,
    /// Whether the changelog was updated
    pub changelog_updated: bool,
}

/// Show the next inferred version based on conventional commits
///
/// # Arguments
/// * `options` - Configuration options (optional)
///
/// # Returns
/// The inferred version and bump type
///
/// # Example
/// ```javascript
/// const { show } = require('@nanodelabs/novalyn');
/// const result = await show({ cwd: '/path/to/repo' });
/// console.log(`Next version: ${result.version} (${result.bumpType})`);
/// ```
#[napi]
pub async fn show(options: Option<JsConfigOptions>) -> Result<JsVersionResult> {
    let opts = options.unwrap_or(JsConfigOptions {
        cwd: None,
        from: None,
        to: None,
        new_version: None,
        exclude_authors: None,
        hide_author_email: None,
        no_authors: None,
        github_token: None,
        no_github_alias: None,
        dry_run: Some(true),
    });

    let mut release_opts: ReleaseOptions = opts.into();
    release_opts.dry_run = true; // Always dry-run for show

    let outcome = run_release(release_opts)
        .map_err(|e| Error::from_reason(format!("Release error: {}", e)))?;

    // Determine bump type by comparing versions
    let bump_type = if let Some(prev) = &outcome.previous {
        if outcome.version.major > prev.major {
            "major"
        } else if outcome.version.minor > prev.minor {
            "minor"
        } else if outcome.version.patch > prev.patch {
            "patch"
        } else {
            "none"
        }
    } else {
        "initial"
    };

    Ok(JsVersionResult {
        version: outcome.version.to_string(),
        bump_type: bump_type.to_string(),
    })
}

/// Generate a changelog block for the commits since the last release
///
/// # Arguments
/// * `options` - Configuration options (optional)
///
/// # Returns
/// The generated markdown and metadata
///
/// # Example
/// ```javascript
/// const { generate } = require('@nanodelabs/novalyn');
/// const result = await generate({ cwd: '/path/to/repo' });
/// console.log(result.markdown);
/// ```
#[napi]
pub async fn generate(options: Option<JsConfigOptions>) -> Result<JsGenerateResult> {
    let opts = options.unwrap_or(JsConfigOptions {
        cwd: None,
        from: None,
        to: None,
        new_version: None,
        exclude_authors: None,
        hide_author_email: None,
        no_authors: None,
        github_token: None,
        no_github_alias: None,
        dry_run: Some(true),
    });

    let mut release_opts: ReleaseOptions = opts.into();
    release_opts.dry_run = true; // Dry-run to just generate

    let outcome = run_release(release_opts)
        .map_err(|e| Error::from_reason(format!("Release error: {}", e)))?;

    // Read the generated markdown from the changelog file (or reconstruct it)
    let markdown = std::fs::read_to_string(&outcome.changelog_path)
        .unwrap_or_else(|_| format!("## v{}\n\nNo changelog generated", outcome.version));

    Ok(JsGenerateResult {
        markdown,
        version: outcome.version.to_string(),
        commit_count: outcome.commit_count as u32,
    })
}

/// Run a full release: infer version, generate changelog, and optionally tag
///
/// # Arguments
/// * `options` - Configuration options (optional)
///
/// # Returns
/// Release result with metadata
///
/// # Example
/// ```javascript
/// const { release } = require('@nanodelabs/novalyn');
/// const result = await release({
///   cwd: '/path/to/repo',
///   newVersion: '1.0.0'
/// });
/// console.log(`Released ${result.version}`);
/// ```
#[napi]
pub async fn release(options: Option<JsConfigOptions>) -> Result<JsReleaseResult> {
    let opts = options.unwrap_or(JsConfigOptions {
        cwd: None,
        from: None,
        to: None,
        new_version: None,
        exclude_authors: None,
        hide_author_email: None,
        no_authors: None,
        github_token: None,
        no_github_alias: None,
        dry_run: None,
    });

    let release_opts: ReleaseOptions = opts.into();

    let outcome = run_release(release_opts)
        .map_err(|e| Error::from_reason(format!("Release error: {}", e)))?;

    Ok(JsReleaseResult {
        version: outcome.version.to_string(),
        commit_count: outcome.commit_count as u32,
        changelog_updated: outcome.wrote,
    })
}

/// Get the current version from Cargo.toml
///
/// # Arguments
/// * `cwd` - Working directory (optional, defaults to current directory)
///
/// # Returns
/// The current version string
#[napi]
pub fn get_current_version(cwd: Option<String>) -> Result<String> {
    let path = cwd
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let cargo_path = path.join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_path)
        .map_err(|e| Error::from_reason(format!("Failed to read Cargo.toml: {}", e)))?;

    let doc: toml_edit::DocumentMut = content
        .parse()
        .map_err(|e| Error::from_reason(format!("Failed to parse Cargo.toml: {}", e)))?;

    let version = doc
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::from_reason("No version found in Cargo.toml"))?;

    Ok(version.to_string())
}
