use crate::config::{ResolvedConfig, SemverImpact, TypeConfigResolved};
use crate::git::RawCommit;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct ParsedCommit {
    pub raw: RawCommit,
    pub r#type: String,
    pub scope: Option<String>,
    pub description: String,
    pub body: String,
    pub footers: Vec<(String, String)>,
    pub breaking: bool,
    pub issues: Vec<u64>,
    pub co_authors: Vec<String>,
    pub type_cfg: Option<TypeConfigResolved>,
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

pub fn parse_and_classify(commits: Vec<RawCommit>, cfg: &ResolvedConfig) -> Vec<ParsedCommit> {
    let rex = Regex::new(r"^(?P<type>[a-zA-Z]+)(\((?P<scope>[^)]+)\))?(?P<bang>!)?: (?P<desc>.+)$")
        .unwrap();
    let mut out = Vec::new();
    for rc in commits {
        let mut p = parse_one(&rc, &rex);
        classify(&mut p, cfg);
        if should_keep(&p) {
            out.push(p);
        }
    }
    out
}

fn parse_one(rc: &RawCommit, rex: &Regex) -> ParsedCommit {
    let mut r#type = String::from("other");
    let mut scope = None;
    let mut description = rc.summary.clone();
    let mut breaking = false;
    if let Some(caps) = rex.captures(&rc.summary) {
        r#type = caps.name("type").unwrap().as_str().to_ascii_lowercase();
        scope = caps.name("scope").map(|m| m.as_str().to_string());
        description = caps.name("desc").unwrap().as_str().to_string();
        if caps.name("bang").is_some() {
            breaking = true;
        }
    }
    let mut body = rc.body.clone();
    let mut footers: Vec<(String, String)> = Vec::new();
    if !body.is_empty() {
        let lines: Vec<&str> = body.lines().collect();
        let mut footer_start = lines.len();
        for i in (0..lines.len()).rev() {
            let line = lines[i];
            if line.trim().is_empty() {
                footer_start = i;
                break;
            }
            if let Some((k, v)) = line.split_once(':') {
                if k.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ' ')
                {
                    footers.push((k.trim().to_string(), v.trim().to_string()));
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        footers.reverse();
        if footer_start < lines.len() {
            body = lines[..footer_start].join("\n");
        }
    }
    for (k, v) in &footers {
        if (k.eq_ignore_ascii_case("BREAKING CHANGE") || k.eq_ignore_ascii_case("BREAKING CHANGES"))
            && (!v.is_empty() || !breaking) {
                breaking = true;
            }
    }
    let mut issues = collect_issue_numbers(&rc.summary);
    issues.extend(collect_issue_numbers(&body));
    for (k, v) in &footers {
        issues.extend(collect_issue_numbers(k));
        issues.extend(collect_issue_numbers(v));
    }
    issues.sort_unstable();
    issues.dedup();
    let mut co_authors = Vec::new();
    for (k, v) in &footers {
        if k.eq_ignore_ascii_case("Co-authored-by") {
            co_authors.push(v.to_string());
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
    }
}

fn classify(pc: &mut ParsedCommit, cfg: &ResolvedConfig) {
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
    if pc.r#type == "chore"
        && pc
            .raw
            .summary
            .to_ascii_lowercase()
            .starts_with("chore(deps)")
        && !pc.breaking
    {
        return false;
    }
    true
}

fn collect_issue_numbers(s: &str) -> Vec<u64> {
    let mut v = Vec::new();
    static ISS_RE: once_cell::sync::Lazy<Regex> =
        once_cell::sync::Lazy::new(|| Regex::new(r"#(\d+)").unwrap());
    for cap in ISS_RE.captures_iter(s) {
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
            new.patch += 1;
            impact = Patch;
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
) -> String {
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
    out
}
