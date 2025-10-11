use crate::config::{ResolvedConfig, SemverImpact, TypeConfigResolved};
use crate::conventional::parse_commit_fast;
use crate::git::RawCommit;
use ecow::{EcoString, EcoVec};
use rayon::prelude::*;

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
    pub index: usize, // original chronological order position for deterministic ordering
}

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

fn parse_and_classify_sequential(
    commits: EcoVec<RawCommit>,
    cfg: &ResolvedConfig,
) -> EcoVec<ParsedCommit> {
    tracing::debug!(
        count = commits.len(),
        mode = "sequential",
        "parsing_commits"
    );
    let mut out = EcoVec::new();
    for (idx, rc) in commits.into_iter().enumerate() {
        let mut p = parse_one(&rc);
        p.index = idx;
        classify(&mut p, cfg);
        if should_keep(&p) {
            tracing::debug!(commit = %p.raw.short_id, r#type = %p.r#type, scope = ?p.scope, breaking = p.breaking, issues = ?p.issues, "classified");
            out.push(p);
        }
    }
    out
}

fn parse_and_classify_parallel(
    commits: EcoVec<RawCommit>,
    cfg: &ResolvedConfig,
) -> EcoVec<ParsedCommit> {
    tracing::debug!(count = commits.len(), mode = "parallel", "parsing_commits");

    // Create indexed commits to preserve original order
    let indexed_commits: Vec<(usize, RawCommit)> = commits.into_iter().enumerate().collect();

    // Process in parallel while maintaining original index
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

    // Log the classified commits (in parallel processing, order may be different in logs)
    for p in &parsed {
        tracing::debug!(commit = %p.raw.short_id, r#type = %p.r#type, scope = ?p.scope, breaking = p.breaking, issues = ?p.issues, "classified");
    }

    // Sort back to original chronological order
    parsed.make_mut().sort_by_key(|p| p.index);
    parsed
}

// Parse a single raw commit using our ultra-fast parser
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
