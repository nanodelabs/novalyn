use crate::config::{ResolvedConfig, SemverImpact, TypeConfigResolved};
use crate::conventional::parse_commit_fast;
use crate::git::RawCommit;
use ecow::{EcoString, EcoVec};
use rayon::prelude::*;

/// A parsed conventional commit with classified type and metadata.
///
/// Contains both the original raw commit data and extracted conventional
/// commit fields like type, scope, breaking changes, and issue references.
#[derive(Debug, Clone)]
pub struct ParsedCommit {
    pub raw: RawCommit,
    pub r#type: EcoString,
    pub scope: Option<EcoString>,
    pub description: EcoString,
    pub body: EcoString,
    pub footers: EcoVec<(EcoString, EcoString)>,
    pub breaking: bool,
    pub issues: EcoVec<u64>,
    pub co_authors: EcoVec<EcoString>,
    pub type_cfg: Option<TypeConfigResolved>,
    /// Original chronological order position for deterministic ordering
    pub index: usize,
}

/// Semantic version bump type inferred from commits.
///
/// Determines how the version number should be incremented based on
/// conventional commit types and breaking changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpKind {
    Major,
    Minor,
    Patch,
    None,
}

impl BumpKind {
    fn escalate(self, other: BumpKind) -> BumpKind {
        use BumpKind::*;
        match (self, other) {
            (Major, _) | (_, Major) => Major,
            (Minor, _) | (_, Minor) => Minor,
            (Patch, _) | (_, Patch) => Patch,
            (None, None) => None,
        }
    }
}

/// Parse and classify commits using either sequential or parallel processing.
///
/// Automatically chooses between sequential and parallel processing based on
/// the number of commits and the `NOVALYN_PARALLEL_THRESHOLD` environment variable.
///
/// # Arguments
/// * `commits` - Raw commits from git repository
/// * `cfg` - Resolved configuration with commit type definitions
///
/// # Returns
/// Parsed and classified commits, preserving chronological order
pub fn parse_and_classify(
    commits: EcoVec<RawCommit>,
    cfg: &ResolvedConfig,
) -> EcoVec<ParsedCommit> {
    let threshold = std::env::var("NOVALYN_PARALLEL_THRESHOLD")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50);

    if commits.len() >= threshold {
        parse_and_classify_parallel(commits, cfg)
    } else {
        parse_and_classify_sequential(commits, cfg)
    }
}

/// Parse and classify commits sequentially in a single thread.
///
/// Used for small commit batches where parallel overhead exceeds benefits.
fn parse_and_classify_sequential(
    commits: EcoVec<RawCommit>,
    cfg: &ResolvedConfig,
) -> EcoVec<ParsedCommit> {
    tracing::debug!(
        count = commits.len(),
        mode = "sequential",
        "parsing_commits"
    );
    use crate::utils::process_indexed;
    process_indexed(commits.into_iter().enumerate(), |idx, rc| {
        let mut p = parse_one(&rc);
        p.index = idx;
        classify(&mut p, cfg);
        if should_keep(&p) {
            tracing::debug!(commit = %p.raw.short_id, r#type = %p.r#type, scope = ?p.scope, breaking = p.breaking, issues = ?p.issues, "classified");
            Some(p)
        } else {
            None
        }
    })
}

/// Parse and classify commits in parallel using rayon.
///
/// Processes commits concurrently while preserving original chronological order.
/// Each commit is parsed and classified independently, then sorted back by index.
fn parse_and_classify_parallel(
    commits: EcoVec<RawCommit>,
    cfg: &ResolvedConfig,
) -> EcoVec<ParsedCommit> {
    tracing::debug!(count = commits.len(), mode = "parallel", "parsing_commits");

    let indexed_commits: Vec<(usize, RawCommit)> = commits.into_iter().enumerate().collect();
    let mut parsed: EcoVec<ParsedCommit> = indexed_commits
        .par_iter()
        .map(|(idx, rc)| {
            let mut p = parse_one(rc);
            p.index = *idx;
            classify(&mut p, cfg);
            p
        })
        .filter(should_keep)
        .collect::<Vec<_>>()
        .into();
    // Sort back to original chronological order
    parsed.make_mut().sort_by_key(|p| p.index);
    parsed
}

/// Parse a single raw commit using our ultra-fast zero-copy parser.
///
/// Delegates to the optimized `parse_commit_fast` function for actual parsing,
/// then wraps the result in a `ParsedCommit` with metadata.
#[inline]
fn parse_one(rc: &RawCommit) -> ParsedCommit {
    let parsed = parse_commit_fast(rc);

    ParsedCommit {
        raw: rc.clone(),
        r#type: parsed.r#type,
        scope: parsed.scope,
        description: parsed.description,
        body: parsed.body,
        footers: parsed.footers,
        breaking: parsed.breaking,
        issues: parsed.issues,
        co_authors: parsed.co_authors,
        type_cfg: None,
        index: 0,
    }
}

/// Classify a parsed commit by matching its type against configured types.
///
/// Sets the `type_cfg` field if a matching type is found in the configuration.
fn classify(pc: &mut ParsedCommit, cfg: &ResolvedConfig) {
    // Apply scope_map if provided (exact match)
    if let Some(sc) = &mut pc.scope {
        if let Some(mapped) = cfg.scope_map.get(sc) {
            if mapped.is_empty() {
                pc.scope = None;
            } else {
                *sc = mapped.clone();
            }
        }
    }
    if let Some(tc) = cfg.types.iter().find(|t| t.key == pc.r#type) {
        if tc.enabled {
            pc.type_cfg = Some(tc.clone());
        }
    }
}

/// Determine if a parsed commit should be kept in the changelog.
///
/// Commits are kept if they have a valid type configuration.
fn should_keep(pc: &ParsedCommit) -> bool {
    if let Some(tc) = &pc.type_cfg {
        if !tc.enabled {
            return false;
        }
    }
    if pc.r#type == "chore" && !pc.breaking {
        // Filter dependency update chores: chore(deps), chore(deps-dev), chore(deps-*) etc.
        // Accept if not starting with chore(deps because there may be other chore scopes we keep
        let lower = pc.raw.summary.to_ascii_lowercase();
        if lower.starts_with("chore(deps") {
            return false;
        }
    }
    true
}

pub fn infer_version(
    previous: &semver::Version,
    commits: &[ParsedCommit],
    override_new: Option<semver::Version>,
) -> (semver::Version, BumpKind) {
    if let Some(v) = override_new {
        return (v, BumpKind::None);
    }
    if commits.is_empty() {
        // No commits at all -> treat as no change (idempotent rerun)
        return (previous.clone(), BumpKind::None);
    }
    use BumpKind::*;
    let mut impact = BumpKind::None;
    for c in commits {
        let bump = if c.breaking {
            Major
        } else if let Some(tc) = &c.type_cfg {
            match tc.semver {
                SemverImpact::Major => Major,
                SemverImpact::Minor => Minor,
                SemverImpact::Patch => Patch,
                SemverImpact::None => None,
            }
        } else {
            None
        };
        impact = impact.escalate(bump);
    }
    let mut new = previous.clone();
    match impact {
        Major => {
            if previous.major == 0 {
                new.minor += 1;
                new.patch = 0;
            } else {
                new.major += 1;
                new.minor = 0;
                new.patch = 0;
            }
        }
        Minor => {
            if previous.major == 0 {
                new.patch += 1;
                impact = Patch; // degrade classification for reporting
            } else {
                new.minor += 1;
                new.patch = 0;
            }
        }
        Patch => {
            new.patch += 1;
        }
        None => {
            // No impactful commits => still bump patch (default policy)
            new.patch += 1;
            return (new, Patch);
        }
    }
    (new, impact)
}

pub fn bump_cargo_version(
    path: &std::path::Path,
    new_version: &semver::Version,
) -> anyhow::Result<()> {
    use anyhow::Context;
    let txt = std::fs::read_to_string(path.join("Cargo.toml"))?;
    let mut doc: toml_edit::DocumentMut = txt.parse().context("parse Cargo.toml")?;
    if let Some(pkg) = doc.get_mut("package") {
        if let Some(ver) = pkg.get_mut("version") {
            *ver = toml_edit::value(new_version.to_string());
        }
    }
    std::fs::write(path.join("Cargo.toml"), doc.to_string())?;
    Ok(())
}

/// Interpolate template variables in a string.
///
/// Supports the following placeholders:
/// - `{{from}}` - Previous version
/// - `{{to}}` - New version  
/// - `{{date}}` - Release date in ISO format
///
/// # Arguments
/// * `template` - Template string with placeholders
/// * `previous` - Previous version
/// * `new_version` - New version
/// * `date` - Release date
///
/// # Returns
/// Interpolated string
pub fn interpolate(
    template: &str,
    previous: &semver::Version,
    new_version: &semver::Version,
    date: &jiff::civil::Date,
) -> EcoString {
    let mut out = String::with_capacity(template.len() + 16);
    let mut chars = template.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' && chars.peek() == Some(&'{') {
            chars.next();
            if chars.peek() == Some(&'{') {
                // not token actually
                out.push(ch);
                continue;
            }
            let mut key = String::new();
            while let Some(&c) = chars.peek() {
                if c == '}' {
                    chars.next();
                    if chars.peek() == Some(&'}') {
                        chars.next();
                        break;
                    } else {
                        key.push(c);
                    }
                } else {
                    key.push(c);
                    chars.next();
                }
            }
            let rep = match key.as_str() {
                "newVersion" => new_version.to_string(),
                "previousVersion" => previous.to_string(),
                "date" => format!("{}-{:02}-{:02}", date.year(), date.month(), date.day()),
                _ => format!("{{{{{}}}}}", key),
            };
            out.push_str(&rep);
        } else {
            out.push(ch);
        }
    }
    out.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_override_used() {
        let prev = semver::Version::parse("1.2.3").unwrap();
        let (v, kind) = infer_version(&prev, &[], Some(semver::Version::parse("9.9.9").unwrap()));
        assert_eq!(v.to_string(), "9.9.9");
        assert_eq!(kind, BumpKind::None);
    }
    #[test]
    fn idempotent_same_version_no_change() {
        let prev = semver::Version::parse("1.2.3").unwrap();
        // No commits -> same version (no change)
        let (v, kind) = infer_version(&prev, &[], None);
        assert_eq!(v.to_string(), "1.2.3");
        assert_eq!(kind, BumpKind::None);
    }
}
