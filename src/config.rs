use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use semver::Version;
use serde::Deserialize;
use tracing::warn;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum TypeToggleOrConfig {
    Disabled(bool), // false means disabled, true treated as default object
    Config(TypeConfigPartial),
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TypeConfigPartial {
    pub title: Option<String>,
    pub emoji: Option<String>,
    pub semver: Option<String>, // major | minor | patch | none
}

#[derive(Debug, Clone)]
pub struct TypeConfigResolved {
    pub key: String,
    pub title: String,
    pub emoji: String,
    pub semver: SemverImpact,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemverImpact {
    Major,
    Minor,
    Patch,
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
    pub new_version: Option<String>,
    #[serde(rename = "types")]
    pub types_override: Option<BTreeMap<String, TypeToggleOrConfig>>, // allow disabling or overriding
    pub scope_map: Option<BTreeMap<String, String>>, // future
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
    pub scope_map: std::collections::BTreeMap<String, String>,
    pub types: Vec<TypeConfigResolved>,
    pub new_version: Option<Version>,
    pub warnings: Vec<String>,
    pub github_token: Option<String>,
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
    pub cwd: &'a Path,
    pub cli_overrides: Option<RawConfig>,
}

pub fn load_config(opts: LoadOptions) -> Result<ResolvedConfig> {
    let mut warnings = Vec::new();
    let mut source_file = None;
    let mut raw_stack: Vec<RawConfig> = Vec::new();

    // defaults placeholder (empty RawConfig means rely on default types below)
    // 1. changelogen.toml
    if let Some(path) = find_file(opts.cwd, "changelogen.toml") {
        match load_file(&path) {
            Ok(rc) => {
                source_file = Some(path.clone());
                raw_stack.push(rc);
            }
            Err(e) => warnings.push(format!("Failed to load changelogen.toml: {e}")),
        }
    }

    // 2. Cargo.toml [package.metadata.changelogen]
    if let Some(cargo_path) = find_file(opts.cwd, "Cargo.toml") {
        match fs::read_to_string(&cargo_path) {
            Ok(s) => {
                if let Some(rc) = extract_metadata_block(&s, &mut warnings) {
                    raw_stack.push(rc);
                }
            }
            Err(e) => warnings.push(format!("Failed to read Cargo.toml: {e}")),
        }
    }

    // 3. CLI overrides last
    if let Some(cli) = opts.cli_overrides {
        raw_stack.push(cli);
    }

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
                                    emoji: String::new(),
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
                Err(e) => warnings.push(format!("Invalid new_version '{vs}': {e}")),
            }
        }
    }

    let github_token = resolve_github_token();

    // accumulate unknown keys warnings (after all layers so later layers can override earlier ones silently)
    for raw in &raw_stack {
        for k in raw._unknown.keys() {
            warnings.push(format!("Unknown config key: {k}"));
        }
    }

    // Attempt repository detection (non-fatal)
    let repo = detect_repository(opts.cwd, &mut warnings);

    // Merge scope_map layering later entries override earlier
    let mut scope_map: BTreeMap<String, String> = BTreeMap::new();
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
        cwd: opts.cwd.to_path_buf(),
        source_file,
        repo,
        scope_map,
    })
}

fn load_file(path: &Path) -> Result<RawConfig> {
    let txt = fs::read_to_string(path).with_context(|| format!("Reading {path:?}"))?;
    let rc: RawConfig =
        toml_edit::de::from_str(&txt).with_context(|| format!("Parsing TOML {path:?}"))?;
    Ok(rc)
}

fn find_file(cwd: &Path, name: &str) -> Option<PathBuf> {
    let candidate = cwd.join(name);
    if candidate.exists() {
        Some(candidate)
    } else {
        None
    }
}

fn extract_metadata_block(cargo_toml: &str, warnings: &mut Vec<String>) -> Option<RawConfig> {
    // parse using toml_edit to avoid losing formatting
    let doc: toml_edit::DocumentMut = match cargo_toml.parse() {
        Ok(d) => d,
        Err(e) => {
            warnings.push(format!("Cargo.toml parse error: {e}"));
            return None;
        }
    };
    if let Some(pkg) = doc.get("package")
        && let Some(meta) = pkg.get("metadata")
        && let Some(cl) = meta.get("changelogen")
    {
        let cl_str = cl.to_string();
        return match toml_edit::de::from_str::<RawConfig>(&cl_str.to_string()) {
            Ok(rc) => {
                // ensure we deserialized a table
                if rc.types_override.is_none() && !cl.is_table() {
                    warnings.push("metadata.changelogen not a table".into());
                }
                Some(rc)
            }
            Err(e) => {
                warnings.push(format!("Failed to parse metadata.changelogen: {e}"));
                None
            }
        };
    }
    None
}

fn resolve_github_token() -> Option<String> {
    for key in ["CHANGELOGEN_TOKENS_GITHUB", "GITHUB_TOKEN", "GH_TOKEN"] {
        if let Ok(v) = std::env::var(key)
            && !v.is_empty()
        {
            return Some(v);
        }
    }
    None
}

pub fn log_warnings(cfg: &ResolvedConfig) {
    for w in &cfg.warnings {
        warn!(target = "changelogen::config", "{w}");
    }
}

fn detect_repository(cwd: &Path, warnings: &mut Vec<String>) -> Option<repo_mod::Repository> {
    // crate path valid when used as library
    // Open git repo; if not a git repository, silently return None (git layer will handle hard error later)
    let repo = match git2::Repository::discover(cwd) {
        Ok(r) => r,
        Err(_) => return None,
    };
    // Preferred remote: origin, else first
    let remotes = match repo.remotes() {
        Ok(r) => r,
        Err(e) => {
            warnings.push(format!("Failed to list git remotes: {e}"));
            return None;
        }
    };
    let mut chosen: Option<String> = None;
    if remotes.iter().any(|n| n == Some("origin"))
        && let Ok(remote) = repo.find_remote("origin")
        && let Some(url) = remote.url()
    {
        chosen = Some(url.to_string());
    }
    if chosen.is_none() {
        for name in remotes.iter().flatten() {
            if let Ok(remote) = repo.find_remote(name)
                && let Some(url) = remote.url()
            {
                chosen = Some(url.to_string());
                break;
            }
        }
    }
    let remote_url = chosen?;
    match repo_mod::Repository::parse(&remote_url) {
        Some(r) => Some(r),
        None => {
            warnings.push(format!("Unrecognized remote URL format: {remote_url}"));
            None
        }
    }
}
