use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use ecow::{EcoString, EcoVec};
use semver::Version;
use serde::Deserialize;
use tracing::warn;

/// Configuration for commit type display and classification.
///
/// Can be either a boolean toggle or a full configuration object.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TypeToggleOrConfig {
    /// false means disabled, true treated as default object
    Disabled(bool),
    /// Full configuration with custom title, emoji, and semver impact
    Config(TypeConfigPartial),
}

/// Partial configuration for a commit type (from TOML).
///
/// All fields are optional to allow incremental configuration.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TypeConfigPartial {
    /// Display title for this commit type
    pub title: Option<EcoString>,
    /// Emoji prefix for changelog entries
    pub emoji: Option<EcoString>,
    /// Semantic version impact: "major" | "minor" | "patch" | "none"
    pub semver: Option<EcoString>,
}

/// Fully resolved configuration for a commit type.
///
/// All fields have concrete values after merging defaults and user config.
#[derive(Debug, Clone)]
pub struct TypeConfigResolved {
    /// Unique identifier for the commit type (e.g., "feat", "fix")
    pub key: EcoString,
    /// Display title for changelog sections
    pub title: EcoString,
    /// Emoji prefix for entries
    pub emoji: EcoString,
    /// Semantic versioning impact
    pub semver: SemverImpact,
    /// Whether this type is enabled for display
    pub enabled: bool,
}

/// Semantic version impact level for a commit type.
///
/// Determines how a commit affects version number incrementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemverImpact {
    /// Breaking changes - increment major version
    Major,
    /// New features - increment minor version
    Minor,
    /// Bug fixes - increment patch version
    Patch,
    /// No version impact
    None,
}

impl SemverImpact {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "major" => Some(Self::Major),
            "minor" => Some(Self::Minor),
            "patch" => Some(Self::Patch),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct RawConfig {
    pub new_version: Option<EcoString>,
    #[serde(rename = "types")]
    pub types_override: Option<BTreeMap<EcoString, TypeToggleOrConfig>>, // allow disabling or overriding
    pub scope_map: Option<BTreeMap<EcoString, EcoString>>, // future
    pub hide_author_email: Option<bool>,
    pub no_authors: Option<bool>,
    // capture unknown keys (flatten) for warning emission
    #[serde(flatten)]
    pub _unknown: BTreeMap<String, serde_json::Value>,
}

// Access repository module via crate root (this crate)
use crate::repository as repo_mod; // binary crate re-exports via main, lib via lib.rs

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    // Optional scope mapping (exact match) applied after parsing
    pub scope_map: std::collections::BTreeMap<EcoString, EcoString>,
    pub types: Vec<TypeConfigResolved>,
    pub new_version: Option<Version>,
    pub warnings: EcoVec<EcoString>,
    pub github_token: Option<EcoString>,
    pub cwd: PathBuf,
    pub source_file: Option<PathBuf>,
    pub repo: Option<repo_mod::Repository>, // set by detection (best-effort)
}

pub fn default_types() -> Vec<TypeConfigResolved> {
    // Mirror JS @unjs/changelogen defaults (order matters)
    let data: &[(&str, &str, &str, SemverImpact)] = &[
        ("feat", "Features", "✨", SemverImpact::Minor),
        ("fix", "Bug Fixes", "🐞", SemverImpact::Patch),
        ("perf", "Performance", "⚡️", SemverImpact::Patch),
        ("docs", "Documentation", "📚", SemverImpact::None),
        ("refactor", "Refactors", "🛠", SemverImpact::Patch),
        ("style", "Styles", "🎨", SemverImpact::None),
        ("test", "Tests", "🧪", SemverImpact::None),
        ("build", "Build System", "📦", SemverImpact::None),
        ("ci", "Continuous Integration", "👷", SemverImpact::None),
        ("chore", "Chores", "🧹", SemverImpact::None),
        ("revert", "Reverts", "⏪", SemverImpact::Patch),
    ];
    data.iter()
        .map(|(k, t, e, s)| TypeConfigResolved {
            key: (*k).into(),
            title: (*t).into(),
            emoji: (*e).into(),
            semver: *s,
            enabled: true,
        })
        .collect()
}

pub struct LoadOptions<'a> {
    /// Working directory to search for config files
    pub cwd: &'a Path,
    /// Optional CLI-provided configuration overrides
    pub cli_overrides: Option<RawConfig>,
}

/// Load and merge configuration from multiple sources asynchronously.
///
/// Configuration precedence (highest to lowest):
/// 1. CLI overrides
/// 2. Cargo.toml [package.metadata.novalyn]
/// 3. novalyn.toml
/// 4. Built-in defaults
///
/// # Arguments
/// * `opts` - Load options specifying paths and overrides
///
/// # Returns
/// * `Ok(ResolvedConfig)` - Fully merged configuration
/// * `Err` - Critical configuration error (warnings stored in config)
pub async fn load_config_async(opts: LoadOptions<'_>) -> Result<ResolvedConfig> {
    let mut warnings = EcoVec::new();
    let mut source_file = None;
    let mut raw_stack: Vec<RawConfig> = Vec::new();

    // Load config files concurrently using join! for parallel I/O
    let novalyn_toml_path = find_file(opts.cwd, "novalyn.toml");
    let cargo_toml_path = find_file(opts.cwd, "Cargo.toml");

    // Load both files concurrently if they exist
    let (novalyn_result, cargo_result) = tokio::join!(
        async {
            if let Some(path) = &novalyn_toml_path {
                Some(load_file_async(path).await)
            } else {
                None
            }
        },
        async {
            if let Some(path) = &cargo_toml_path {
                Some(tokio::fs::read_to_string(path).await)
            } else {
                None
            }
        }
    );

    // 1. novalyn.toml
    if let Some(result) = novalyn_result {
        match result {
            Ok(rc) => {
                source_file = Some(novalyn_toml_path.unwrap());
                raw_stack.push(rc);
            }
            Err(e) => {
                warnings.push(format!("Failed loading novalyn.toml: {e}").into());
            }
        }
    }

    // 2. Cargo.toml [package.metadata.novalyn]
    if let Some(result) = cargo_result {
        match result {
            Ok(s) => {
                if let Some(rc) = extract_metadata_block(&s, &mut warnings) {
                    raw_stack.push(rc);
                }
            }
            Err(e) => {
                warnings.push(format!("Failed loading Cargo.toml: {e}").into());
            }
        }
    }

    // 3. CLI overrides last
    if let Some(cli) = opts.cli_overrides {
        raw_stack.push(cli);
    }

    // Call common merge logic
    merge_and_resolve_config(opts.cwd, raw_stack, warnings, source_file)
}

/// Merge and resolve configuration from raw config stack.
///
/// This is the common logic used by both sync and async config loaders.
fn merge_and_resolve_config(
    cwd: &Path,
    raw_stack: Vec<RawConfig>,
    mut warnings: EcoVec<EcoString>,
    source_file: Option<PathBuf>,
) -> Result<ResolvedConfig> {
    // Merge stack in order added (file(s) then CLI). Defaults applied separately.
    let mut types = default_types();

    for raw in &raw_stack {
        if let Some(map) = &raw.types_override {
            for (k, v) in map {
                // locate or append
                let idx = types.iter().position(|t| &t.key == k);
                match v {
                    TypeToggleOrConfig::Disabled(b) => {
                        if !*b {
                            // false disables
                            if let Some(i) = idx {
                                types[i].enabled = false;
                            } else {
                                // create disabled placeholder so later override could re-enable
                                types.push(TypeConfigResolved {
                                    key: k.clone(),
                                    title: k.clone(),
                                    emoji: EcoString::new(),
                                    semver: SemverImpact::None,
                                    enabled: false,
                                });
                            }
                        }
                    }
                    TypeToggleOrConfig::Config(part) => {
                        let semver = part
                            .semver
                            .as_deref()
                            .and_then(SemverImpact::from_str)
                            .unwrap_or_else(|| {
                                idx.map(|i| types[i].semver).unwrap_or(SemverImpact::None)
                            });
                        if let Some(i) = idx {
                            let t = &mut types[i];
                            if let Some(title) = &part.title {
                                t.title = title.clone();
                            }
                            if let Some(emoji) = &part.emoji {
                                t.emoji = emoji.clone();
                            }
                            t.semver = semver;
                            t.enabled = true;
                        } else {
                            types.push(TypeConfigResolved {
                                key: k.clone(),
                                title: part.title.clone().unwrap_or_else(|| k.clone()),
                                emoji: part.emoji.clone().unwrap_or_default(),
                                semver,
                                enabled: true,
                            });
                        }
                    }
                }
            }
        }
    }

    // new_version validation (take last one provided)
    let mut new_version: Option<Version> = None;
    for raw in &raw_stack {
        if let Some(vs) = &raw.new_version {
            match Version::parse(vs) {
                Ok(v) => new_version = Some(v),
                Err(e) => warnings.push(format!("Invalid new_version '{vs}': {e}").into()),
            }
        }
    }

    let github_token = resolve_github_token();

    // accumulate unknown keys warnings (after all layers so later layers can override earlier ones silently)
    for raw in &raw_stack {
        for k in raw._unknown.keys() {
            warnings.push(format!("Unknown config key: {k}").into());
        }
    }

    // Attempt repository detection (non-fatal)
    let repo = detect_repository(cwd, &mut warnings);

    // Merge scope_map layering later entries override earlier
    let mut scope_map: BTreeMap<EcoString, EcoString> = BTreeMap::new();
    for raw in &raw_stack {
        if let Some(sm) = &raw.scope_map {
            for (k, v) in sm {
                scope_map.insert(k.clone(), v.clone());
            }
        }
    }

    Ok(ResolvedConfig {
        types,
        new_version,
        warnings,
        github_token,
        cwd: cwd.to_path_buf(),
        source_file,
        repo,
        scope_map,
    })
}

/// Load and merge configuration from multiple sources synchronously.
///
/// Configuration precedence (highest to lowest):
/// 1. CLI overrides
/// 2. Cargo.toml [package.metadata.novalyn]
/// 3. novalyn.toml
/// 4. Built-in defaults
///
/// # Arguments
/// * `opts` - Load options specifying paths and overrides
///
/// # Returns
/// * `Ok(ResolvedConfig)` - Fully merged configuration
/// * `Err` - Critical configuration error (warnings stored in config)
pub fn load_config(opts: LoadOptions) -> Result<ResolvedConfig> {
    let mut warnings = EcoVec::new();
    let mut source_file = None;
    let mut raw_stack: Vec<RawConfig> = Vec::new();

    // defaults placeholder (empty RawConfig means rely on default types below)
    // 1. novalyn.toml
    if let Some(path) = find_file(opts.cwd, "novalyn.toml") {
        match load_file(&path) {
            Ok(rc) => {
                source_file = Some(path.clone());
                raw_stack.push(rc);
            }
            Err(e) => warnings.push(format!("Failed to load novalyn.toml: {e}").into()),
        }
    }

    // 2. Cargo.toml [package.metadata.novalyn]
    if let Some(cargo_path) = find_file(opts.cwd, "Cargo.toml") {
        match fs::read_to_string(&cargo_path) {
            Ok(s) => {
                if let Some(rc) = extract_metadata_block(&s, &mut warnings) {
                    raw_stack.push(rc);
                }
            }
            Err(e) => warnings.push(format!("Failed to read Cargo.toml: {e}").into()),
        }
    }

    // 3. CLI overrides last
    if let Some(cli) = opts.cli_overrides {
        raw_stack.push(cli);
    }

    // Call common merge logic
    merge_and_resolve_config(opts.cwd, raw_stack, warnings, source_file)
}

/// Load a TOML configuration file asynchronously.
///
/// # Arguments
/// * `path` - Path to TOML file
///
/// # Returns
/// Parsed configuration or error with context
async fn load_file_async(path: &Path) -> Result<RawConfig> {
    let txt = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("Reading {path:?}"))?;
    let rc: RawConfig =
        toml_edit::de::from_str(&txt).with_context(|| format!("Parsing TOML {path:?}"))?;
    Ok(rc)
}

/// Load a TOML configuration file synchronously (for backward compatibility).
///
/// # Arguments
/// * `path` - Path to TOML file
///
/// # Returns
/// Parsed configuration or error with context
fn load_file(path: &Path) -> Result<RawConfig> {
    let txt = fs::read_to_string(path).with_context(|| format!("Reading {path:?}"))?;
    let rc: RawConfig =
        toml_edit::de::from_str(&txt).with_context(|| format!("Parsing TOML {path:?}"))?;
    Ok(rc)
}

/// Find a configuration file in the given directory.
///
/// # Arguments
/// * `cwd` - Directory to search
/// * `name` - Filename to look for
///
/// # Returns
/// Full path if file exists, None otherwise
fn find_file(cwd: &Path, name: &str) -> Option<PathBuf> {
    let candidate = cwd.join(name);
    if candidate.exists() {
        Some(candidate)
    } else {
        None
    }
}

/// Extract [package.metadata.novalyn] block from Cargo.toml.
///
/// # Arguments
/// * `cargo_toml` - Cargo.toml file content
/// * `warnings` - Vector to append warnings to
///
/// # Returns
/// Parsed configuration if present, None if not found or invalid
fn extract_metadata_block(cargo_toml: &str, warnings: &mut EcoVec<EcoString>) -> Option<RawConfig> {
    // parse using toml_edit to avoid losing formatting
    let doc: toml_edit::DocumentMut = match cargo_toml.parse() {
        Ok(d) => d,
        Err(e) => {
            warnings.push(format!("Cargo.toml parse error: {e}").into());
            return None;
        }
    };
    if let Some(pkg) = doc.get("package")
        && let Some(meta) = pkg.get("metadata")
        && let Some(cl) = meta.get("novalyn")
    {
        let cl_str = cl.to_string();
        return match toml_edit::de::from_str::<RawConfig>(&cl_str.to_string()) {
            Ok(rc) => {
                // ensure we deserialized a table
                if rc.types_override.is_none() && !cl.is_table() {
                    warnings.push("metadata.novalyn not a table".into());
                }
                Some(rc)
            }
            Err(e) => {
                warnings.push(format!("Failed to parse metadata.novalyn: {e}").into());
                None
            }
        };
    }
    None
}

/// Resolve GitHub token from environment variables.
///
/// Checks in order: CHANGELOGEN_TOKENS_GITHUB, GITHUB_TOKEN, GH_TOKEN
///
/// # Returns
/// Token if found in environment, None otherwise
fn resolve_github_token() -> Option<EcoString> {
    for key in ["NOVALYN_TOKENS_GITHUB", "GITHUB_TOKEN", "GH_TOKEN"] {
        if let Ok(v) = std::env::var(key)
            && !v.is_empty()
        {
            return Some(v.into());
        }
    }
    None
}

/// Log configuration warnings using the tracing framework.
///
/// # Arguments
/// * `cfg` - Configuration containing warnings to log
pub fn log_warnings(cfg: &ResolvedConfig) {
    for w in &cfg.warnings {
        warn!(target = "novalyn::config", "{w}");
    }
}

/// Attempt to detect git repository information.
///
/// Tries to parse remote URL and detect repository provider (GitHub, GitLab, etc.)
///
/// # Arguments
/// * `cwd` - Directory to search for repository
/// * `warnings` - Vector to append warnings to
///
/// # Returns
/// Repository information if detected, None otherwise
fn detect_repository(cwd: &Path, warnings: &mut EcoVec<EcoString>) -> Option<repo_mod::Repository> {
    // crate path valid when used as library
    // Open git repo; if not a git repository, silently return None (git layer will handle hard error later)
    let repo = match gix::open(cwd) {
        Ok(r) => r,
        Err(_) => return None,
    };
    // Preferred remote: origin, else first

    // FIX: gix::Repository::remote_names() returns BTreeSet<Cow<'_, BStr>>, not Result
    // So we should use:
    let remotes = repo.remote_names();
    let mut chosen: Option<String> = None;
    // Look for "origin" remote first
    if remotes.iter().any(|name| name.as_ref() == b"origin") {
        if let Ok(remote) = repo.find_remote("origin") {
            if let Some(url) = remote.url(gix::remote::Direction::Fetch) {
                chosen = Some(url.to_string());
            }
        }
    }
    // Fallback: use first available remote
    if chosen.is_none() {
        for name in remotes.iter() {
            if let Ok(remote) = repo.find_remote(name.as_ref()) {
                if let Some(url) = remote.url(gix::remote::Direction::Fetch) {
                    chosen = Some(url.to_string());
                    break;
                }
            }
        }
    }
    let remote_url = chosen?;
    match repo_mod::Repository::parse(&remote_url) {
        Some(r) => Some(r),
        None => {
            warnings.push(format!("Unrecognized remote URL format: {remote_url}").into());
            None
        }
    }
}
