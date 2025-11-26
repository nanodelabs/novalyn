//! Ultra-performant conventional commit parser
//!
//! Hand-optimized zero-copy parser for conventional commits.
//! Directly integrated with ParsedCommit for maximum performance.

use crate::git::RawCommit;
use ecow::{EcoString, EcoVec};

/// Parsed commit fields ready for ParsedCommit construction
pub struct ParsedFields {
    pub r#type: EcoString,
    pub scope: Option<EcoString>,
    pub description: EcoString,
    pub body: EcoString,
    pub footers: EcoVec<(EcoString, EcoString)>,
    pub breaking: bool,
    pub issues: EcoVec<u64>,
    pub co_authors: EcoVec<EcoString>,
}

/// Parse a commit directly into the required fields for ParsedCommit
///
/// This skips intermediate allocations and parses everything in one pass.
#[inline]
pub fn parse_commit_fast(rc: &RawCommit) -> ParsedFields {
    let bytes = rc.summary.as_bytes();
    let mut pos = 0;

    // Parse type - alphanumeric only
    let type_start = pos;
    while pos < bytes.len() && bytes[pos].is_ascii_alphabetic() {
        pos += 1;
    }

    let r#type: EcoString = if pos > type_start {
        rc.summary[type_start..pos].to_ascii_lowercase().into()
    } else {
        "other".into()
    };

    // Check for scope
    let scope = if pos < bytes.len() && bytes[pos] == b'(' {
        pos += 1; // skip '('
        if let Some(offset) = memchr::memchr(b')', &bytes[pos..]) {
            let scope_end = pos + offset;
            let scope_text: EcoString = rc.summary[pos..scope_end].into();
            pos = scope_end + 1; // skip ')'
            Some(scope_text)
        } else {
            None
        }
    } else {
        None
    };

    // Check for breaking change marker
    let mut breaking = if pos < bytes.len() && bytes[pos] == b'!' {
        pos += 1;
        true
    } else {
        false
    };

    // Expect ':'
    if pos < bytes.len() && bytes[pos] == b':' {
        pos += 1;
    }

    // Skip whitespace after ':'
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }

    // Trim all whitespace from both ends (handles edge cases like vertical tabs)
    let description: EcoString = rc.summary[pos..].trim().into();

    // Fast path: no body means no footers
    if rc.body.is_empty() {
        let issues = extract_issues_fast(&rc.summary);
        return ParsedFields {
            r#type,
            scope,
            description,
            body: EcoString::new(),
            footers: EcoVec::new(),
            breaking,
            issues,
            co_authors: EcoVec::new(),
        };
    }

    // Parse body and footers in one pass
    let body_str = rc.body.as_str();
    let lines: Vec<&str> = body_str.lines().collect();

    if lines.is_empty() {
        let issues = extract_issues_fast(&rc.summary);
        return ParsedFields {
            r#type,
            scope,
            description,
            body: EcoString::new(),
            footers: EcoVec::new(),
            breaking,
            issues,
            co_authors: EcoVec::new(),
        };
    }

    // Find footer boundary by scanning backwards
    let mut footer_start_idx = None;
    for i in (0..lines.len()).rev() {
        if lines[i].trim().is_empty() {
            // Check if everything after this looks like footers
            // (footers or continuation lines)
            let mut all_footers = true;
            for &line in &lines[i + 1..] {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    // Check if it's a continuation line (starts with whitespace)
                    if line.starts_with(' ') || line.starts_with('\t') {
                        continue; // continuation line is valid
                    }

                    // Otherwise must be a footer with a colon
                    if let Some(colon_pos) = memchr::memchr(b':', trimmed.as_bytes()) {
                        if colon_pos == 0 || !is_valid_footer_token(&trimmed[..colon_pos]) {
                            all_footers = false;
                            break;
                        }
                    } else {
                        all_footers = false;
                        break;
                    }
                }
            }

            if all_footers && i + 1 < lines.len() {
                footer_start_idx = Some(i + 1);
                break;
            }
        }
    }

    // Check if entire body is footers
    if footer_start_idx.is_none() && !lines.is_empty() {
        let first_trimmed = lines[0].trim();
        if let Some(colon_pos) = memchr::memchr(b':', first_trimmed.as_bytes())
            && colon_pos > 0
            && is_valid_footer_token(&first_trimmed[..colon_pos])
        {
            footer_start_idx = Some(0);
        }
    }

    let footer_start_idx = match footer_start_idx {
        Some(idx) => idx,
        None => {
            // No footers - extract issues from summary and body
            let mut issues = extract_issues_fast(&rc.summary);
            issues.extend(extract_issues_fast(body_str));

            // Convert to Vec for dedup, then back
            let mut issues_vec: Vec<u64> = issues.into_iter().collect();
            issues_vec.sort_unstable();
            issues_vec.dedup();
            let issues: EcoVec<u64> = issues_vec.into();

            return ParsedFields {
                r#type,
                scope,
                description,
                body: body_str.trim().into(),
                footers: EcoVec::new(),
                breaking,
                issues,
                co_authors: EcoVec::new(),
            };
        }
    };

    // Split body
    let body_lines = if footer_start_idx > 0 {
        let body_end = if lines[footer_start_idx - 1].trim().is_empty() {
            footer_start_idx - 1
        } else {
            footer_start_idx
        };
        &lines[..body_end]
    } else {
        &[]
    };

    let body: EcoString = if body_lines.is_empty() {
        EcoString::new()
    } else {
        lines[..body_lines.len()].join("\n").trim().into()
    };

    // Parse footers efficiently
    let mut footers = EcoVec::new();
    let mut co_authors = EcoVec::new();
    let mut current_token: Option<EcoString> = None;
    let mut current_value = EcoString::new();

    for &line in &lines[footer_start_idx..] {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Check if this is a new footer using memchr
        if let Some(colon_pos) = memchr::memchr(b':', trimmed.as_bytes())
            && colon_pos > 0
        {
            let token = &trimmed[..colon_pos];

            if is_valid_footer_token(token) {
                // Save previous footer
                if let Some(tok) = current_token.take() {
                    // Check for breaking change
                    if !breaking
                        && (tok.eq_ignore_ascii_case("BREAKING CHANGE")
                            || tok.eq_ignore_ascii_case("BREAKING-CHANGE")
                            || tok.eq_ignore_ascii_case("BREAKING CHANGES"))
                    {
                        breaking = true;
                    }

                    // Check for co-author
                    if tok.eq_ignore_ascii_case("Co-authored-by") {
                        co_authors.push(current_value.clone());
                    }

                    footers.push((tok, std::mem::take(&mut current_value)));
                }

                let value = trimmed[colon_pos + 1..].trim_start();
                current_token = Some(token.trim().into());
                current_value = value.into();
                continue;
            }
        }

        // Continuation line
        if current_token.is_some() && (line.starts_with(' ') || line.starts_with('\t')) {
            if !current_value.is_empty() {
                current_value = format!("{}\n{}", current_value, trimmed).into();
            } else {
                current_value = trimmed.into();
            }
        }
    }

    // Save last footer
    if let Some(tok) = current_token {
        if !breaking
            && (tok.eq_ignore_ascii_case("BREAKING CHANGE")
                || tok.eq_ignore_ascii_case("BREAKING-CHANGE")
                || tok.eq_ignore_ascii_case("BREAKING CHANGES"))
        {
            breaking = true;
        }

        if tok.eq_ignore_ascii_case("Co-authored-by") {
            co_authors.push(current_value.clone());
        }

        footers.push((tok, current_value));
    }

    // Extract issues from all fields using SIMD-optimized search
    let mut issues = extract_issues_fast(&rc.summary);
    if !body.is_empty() {
        issues.extend(extract_issues_fast(&body));
    }
    for (k, v) in &footers {
        issues.extend(extract_issues_fast(k));
        issues.extend(extract_issues_fast(v));
    }

    // Convert to Vec for dedup, then back
    let mut issues_vec: Vec<u64> = issues.into_iter().collect();
    issues_vec.sort_unstable();
    issues_vec.dedup();
    let issues: EcoVec<u64> = issues_vec.into();

    ParsedFields {
        r#type,
        scope,
        description,
        body,
        footers,
        breaking,
        issues,
        co_authors,
    }
}

/// Fast issue number extraction using memchr
#[inline]
fn extract_issues_fast(text: &str) -> EcoVec<u64> {
    let bytes = text.as_bytes();
    let mut issues = EcoVec::new();
    let mut pos = 0;

    while pos < bytes.len() {
        // Find next '#' using SIMD
        if let Some(offset) = memchr::memchr(b'#', &bytes[pos..]) {
            pos += offset + 1;

            // Parse digits after '#'
            let start = pos;
            while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                pos += 1;
            }

            if pos > start {
                // Safety: we know this is ASCII digits
                if let Ok(num) = text[start..pos].parse::<u64>() {
                    issues.push(num);
                }
            }
        } else {
            break;
        }
    }

    issues
}

/// Check if a string is a valid footer token
#[inline]
fn is_valid_footer_token(token: &str) -> bool {
    !token.is_empty()
        && token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b' ')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_commit(summary: &str, body: &str) -> RawCommit {
        RawCommit {
            id: "test".into(),
            short_id: "test".into(),
            summary: summary.into(),
            body: body.into(),
            author_name: "Author".into(),
            author_email: "author@test.com".into(),
            timestamp: 0,
        }
    }

    #[test]
    fn test_simple_commit() {
        let rc = make_commit("feat: add feature", "");
        let parsed = parse_commit_fast(&rc);

        assert_eq!(parsed.r#type, "feat");
        assert_eq!(parsed.scope, None);
        assert_eq!(parsed.description, "add feature");
        assert_eq!(parsed.body, "");
        assert!(parsed.footers.is_empty());
        assert!(!parsed.breaking);
        assert!(parsed.issues.is_empty());
        assert!(parsed.co_authors.is_empty());
    }

    #[test]
    fn test_with_scope() {
        let rc = make_commit("fix(api): handle null", "");
        let parsed = parse_commit_fast(&rc);

        assert_eq!(parsed.r#type, "fix");
        assert_eq!(parsed.scope, Some("api".into()));
    }

    #[test]
    fn test_breaking() {
        let rc = make_commit("feat!: breaking change", "");
        let parsed = parse_commit_fast(&rc);

        assert!(parsed.breaking);
    }

    #[test]
    fn test_with_issues() {
        let rc = make_commit("feat: add feature #123 #456", "Body with #789");
        let parsed = parse_commit_fast(&rc);

        assert_eq!(parsed.issues.len(), 3);
        assert!(parsed.issues.contains(&123));
        assert!(parsed.issues.contains(&456));
        assert!(parsed.issues.contains(&789));
    }

    #[test]
    fn test_with_footers() {
        let rc = make_commit(
            "feat: add feature",
            "Body text.\n\nReviewed-by: John\nRefs: #123",
        );
        let parsed = parse_commit_fast(&rc);

        assert_eq!(parsed.body, "Body text.");
        assert_eq!(parsed.footers.len(), 2);
        assert_eq!(parsed.footers[0].0, "Reviewed-by");
        assert_eq!(parsed.footers[0].1, "John");
    }

    #[test]
    fn test_breaking_footer() {
        let rc = make_commit("feat: add", "BREAKING CHANGE: breaks stuff");
        let parsed = parse_commit_fast(&rc);

        assert!(parsed.breaking);
    }

    #[test]
    fn test_co_author() {
        let rc = make_commit("feat: add", "Co-authored-by: Jane Doe <jane@example.com>");
        let parsed = parse_commit_fast(&rc);

        assert_eq!(parsed.co_authors.len(), 1);
        assert_eq!(parsed.co_authors[0], "Jane Doe <jane@example.com>");
    }
}
