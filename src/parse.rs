use crate::config::{ResolvedConfig, SemverImpact, TypeConfigResolved};
use crate::git::RawCommit;
use ecow::{EcoString, EcoVec};
use git_conventional::Commit as ConventionalCommit;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;

// Compile-time regex patterns
static COMMIT_HEADER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<type>[a-zA-Z]+)(\((?P<scope>[^)]+)\))?(?P<bang>!)?: (?P<desc>.+)$")
        .expect("Invalid regex pattern for commit header")
});

static ISSUE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"#(\d+)").expect("Invalid regex pattern for issues"));

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
    let threshold = std::env::var("CHANGELOGEN_PARALLEL_THRESHOLD")
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
        let mut p = parse_one(&rc, &COMMIT_HEADER_RE);
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
            let mut p = parse_one(rc, &COMMIT_HEADER_RE);
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

// Parse a single raw commit. Try git-conventional first for richer parsing, fallback to regex.
fn parse_one(rc: &RawCommit, rex: &Regex) -> ParsedCommit {
    let mut r#type = EcoString::from("other");
    let mut scope = None;
    let mut description = rc.summary.clone();
    let mut breaking = false;
    // Attempt conventional commit parse
    if let Ok(cc) = ConventionalCommit::parse(&rc.summary) {
        r#type = cc.type_().as_str().to_ascii_lowercase().into();
        scope = cc.scope().map(|s| s.as_str().into());
        description = cc.description().into();
        if cc.breaking() {
            breaking = true;
        }
    } else if let Some(caps) = rex.captures(&rc.summary) {
        // These unwraps are safe because capture groups are guaranteed by successful regex match
        r#type = caps
            .name("type")
            .map(|m| m.as_str().to_ascii_lowercase().into())
            .unwrap_or_else(|| "other".into());
        scope = caps.name("scope").map(|m| m.as_str().into());
        description = caps
            .name("desc")
            .map(|m| m.as_str().into())
            .unwrap_or_else(|| rc.summary.clone());
        if caps.name("bang").is_some() {
            breaking = true;
        }
    }
    let mut body = rc.body.clone();
    let mut footers: EcoVec<(EcoString, EcoString)> = EcoVec::new();
    if !body.is_empty() {
        let lines: Vec<&str> = body.lines().collect();
        // Parse footers forward: find last blank line; everything after that that matches footer syntax.
        let mut split_idx = None;
        for (idx, line) in lines.iter().enumerate().rev() {
            if line.trim().is_empty() {
                split_idx = Some(idx);
                break;
            }
        }
        let start_footer = split_idx.map(|i| i + 1).unwrap_or(lines.len());
        // Collect raw footer lines (with continuation support)
        let mut cur_key: Option<EcoString> = None;
        let mut cur_val = String::new();
        for &line in &lines[start_footer..] {
            if let Some((k, v)) = line.split_once(':') {
                let k_trim = k.trim();
                if k_trim
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ' ')
                {
                    // flush previous
                    if let Some(k_existing) = cur_key.take() {
                        footers.push((k_existing, cur_val.trim_end().into()));
                        cur_val = String::new();
                    }
                    cur_key = Some(k_trim.into());
                    cur_val.push_str(v.trim_start());
                } else {
                    // invalid footer key -> stop parsing further footers
                    cur_key = None; // discard current
                    break;
                }
            } else if (line.starts_with(' ') || line.starts_with('\t')) && cur_key.is_some() {
                let trimmed = line.trim_start();
                if !trimmed.is_empty() {
                    if !cur_val.is_empty() {
                        cur_val.push('\n');
                    }
                    cur_val.push_str(trimmed);
                }
            } else if line.trim().is_empty() {
                // ignore extra blanks inside footer section
                continue;
            } else {
                // Non-footer pattern terminates footer parsing
                break;
            }
        }
        if let Some(k) = cur_key.take() {
            footers.push((k, cur_val.trim_end().into()));
        }
        if !footers.is_empty() {
            body = if start_footer == lines.len() {
                body
            } else if let Some(idx) = split_idx {
                lines[..idx].join("\n").into()
            } else {
                // This branch should not be reached given the logic above,
                // but we handle it gracefully
                body
            };
        }
    }
    for (k, v) in &footers {
        if (k.eq_ignore_ascii_case("BREAKING CHANGE") || k.eq_ignore_ascii_case("BREAKING CHANGES"))
            && (!v.is_empty() || !breaking)
        {
            breaking = true;
        }
    }
    // If body ended up including trailing blank due to cut logic, trim newline artifacts
    body = body.trim_end().into();
    let mut issues = collect_issue_numbers(&rc.summary);
    issues.extend(collect_issue_numbers(&body));
    for (k, v) in &footers {
        issues.extend(collect_issue_numbers(k));
        issues.extend(collect_issue_numbers(v));
    }
    let mut issues_vec = issues.into_iter().collect::<Vec<_>>();
    issues_vec.sort_unstable();
    issues_vec.dedup();
    issues = issues_vec.into();
    let mut co_authors = EcoVec::new();
    for (k, v) in &footers {
        if k.eq_ignore_ascii_case("Co-authored-by") {
            co_authors.push(v.clone());
        }
    }
    ParsedCommit {
        raw: rc.clone(),
        r#type,
        scope,
        description,
        body,
        footers,
        breaking,
        issues,
        co_authors,
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

fn collect_issue_numbers(s: &str) -> EcoVec<u64> {
    // Capture individual #123 plus grouped variants inside parentheses or separated by commas/spaces.
    // Strategy: first find all #\d+ tokens.
    let mut v = EcoVec::new();
    for cap in ISSUE_RE.captures_iter(s) {
        if let Ok(num) = cap[1].parse() {
            v.push(num);
        }
    }
    v
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
