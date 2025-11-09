//! Property-based tests for conventional commit parsing
//!
//! Tests that the parser handles arbitrary inputs correctly and maintains invariants.

use novalyn_core::conventional::parse_commit_fast;
use novalyn_core::git::RawCommit;
use proptest::prelude::*;

// Strategy for generating valid conventional commit types
fn valid_type() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z]{2,10}").unwrap()
}

// Strategy for generating valid scopes (alphanumeric and dashes)
fn valid_scope() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z0-9-]{1,20}").unwrap()
}

// Strategy for generating descriptions (arbitrary text including whitespace-only)
fn description() -> impl Strategy<Value = String> {
    prop::string::string_regex(".{0,100}").unwrap()
}

// Strategy for generating issue numbers
fn issue_number() -> impl Strategy<Value = u64> {
    1u64..1000000u64
}

proptest! {
    /// Test that parsing never panics on arbitrary input
    #[test]
    fn parse_never_panics(summary in "\\PC*", body in "\\PC*") {
        let commit = RawCommit {
            id: "abc123def456".into(),
            short_id: "abc123".into(),
            summary: summary.into(),
            body: body.into(),
            author_name: "Test".into(),
            author_email: "test@test.com".into(),
            timestamp: 1704067200,
        };
        let _ = parse_commit_fast(&commit);
    }

    /// Test that well-formed commits are parsed correctly
    #[test]
    fn parse_well_formed_commits(
        r#type in valid_type(),
        scope in proptest::option::of(valid_scope()),
        desc in description(),
        breaking in proptest::bool::ANY,
    ) {
        let breaking_marker = if breaking { "!" } else { "" };
        let scope_str = scope.as_ref().map(|s| format!("({})", s)).unwrap_or_default();
        let summary = format!("{}{}{}: {}", r#type, scope_str, breaking_marker, desc);

        let commit = RawCommit {
            id: "abc123def456".into(),
            short_id: "abc123".into(),
            summary: summary.into(),
            body: "".into(),
            author_name: "Test".into(),
            author_email: "test@test.com".into(),
            timestamp: 1704067200.into(),
        };

        let parsed = parse_commit_fast(&commit);

        // Verify type is extracted and lowercased
        assert_eq!(parsed.r#type.as_str(), r#type.to_ascii_lowercase());

        // Verify scope is extracted if present
        assert_eq!(parsed.scope.as_ref().map(|s| s.as_str()), scope.as_deref());

        // Verify breaking flag matches
        assert_eq!(parsed.breaking, breaking);

        // Verify description is extracted (may be empty after trimming whitespace-only input)
        // The description should match the trimmed version of what we put in
        assert_eq!(parsed.description.as_str(), desc.trim());
    }

    /// Test that type extraction handles edge cases
    #[test]
    fn type_extraction_invariants(summary in "\\PC{0,200}") {
        let commit = RawCommit {
            id: "abc123def456".into(),
            short_id: "abc123".into(),
            summary: summary.into(),
            body: "".into(),
            author_name: "Test".into(),
            author_email: "test@test.com".into(),
            timestamp: 1704067200.into(),
        };

        let parsed = parse_commit_fast(&commit);

        // Type should never be empty - defaults to "other"
        assert!(!parsed.r#type.is_empty());

        // Type should always be lowercase
        assert_eq!(parsed.r#type.to_lowercase(), parsed.r#type.as_str());
    }

    /// Test that issue extraction handles various formats
    #[test]
    fn issue_extraction_formats(
        prefix in prop::sample::select(vec!["#", "closes #", "fixes #", "resolves #"]),
        issue_num in issue_number(),
    ) {
        let summary_text = format!("feat: add feature {}{}", prefix, issue_num);
        let commit = RawCommit {
            id: "abc123def456".into(),
            short_id: "abc123".into(),
            summary: summary_text.clone().into(),
            body: "".into(),
            author_name: "Test".into(),
            author_email: "test@test.com".into(),
            timestamp: 1704067200.into(),
        };

        let parsed = parse_commit_fast(&commit);

        // Should extract at least one issue if prefix contains '#'
        if prefix.contains('#') {
            assert!(!parsed.issues.is_empty(), "Failed to extract issue from: {}", summary_text);
            assert!(parsed.issues.contains(&issue_num),
                    "Should contain issue {} in {:?} from {}", issue_num, parsed.issues, summary_text);
        }
    }

    /// Test that footers are parsed from body
    #[test]
    fn footer_parsing(
        key in "[A-Z][a-z-]+",
        value in "\\PC{1,50}",
    ) {
        let body = format!("{}: {}", key, value);
        let commit = RawCommit {
            id: "abc123def456".into(),
            short_id: "abc123".into(),
            summary: "feat: test".into(),
            body: body.into(),
            author_name: "Test".into(),
            author_email: "test@test.com".into(),
            timestamp: 1704067200.into(),
        };

        let parsed = parse_commit_fast(&commit);

        // Known footer keys should be extracted
        let footer_keys = ["BREAKING-CHANGE", "BREAKING CHANGE", "Closes", "Fixes", "Resolves", "Co-authored-by"];
        if footer_keys.iter().any(|&k| key == k) {
            // Should have parsed the footer or detected breaking change
            assert!(parsed.footers.len() > 0 || parsed.breaking || parsed.issues.len() > 0 || parsed.co_authors.len() > 0);
        }
    }

    /// Test that parser handles multiple lines in body
    #[test]
    fn multiline_body_handling(lines in prop::collection::vec("\\PC{0,80}", 0..10)) {
        let body = lines.join("\n");
        let commit = RawCommit {
            id: "abc123def456".into(),
            short_id: "abc123".into(),
            summary: "feat: test".into(),
            body: body.into(),
            author_name: "Test".into(),
            author_email: "test@test.com".into(),
            timestamp: 1704067200.into(),
        };

        // Should not panic
        let _ = parse_commit_fast(&commit);
    }

    /// Test that breaking change detection is consistent
    #[test]
    fn breaking_change_consistency(has_bang in proptest::bool::ANY, has_footer in proptest::bool::ANY) {
        let summary = if has_bang {
            "feat!: breaking change"
        } else {
            "feat: normal change"
        };

        let body = if has_footer {
            "BREAKING CHANGE: this breaks stuff"
        } else {
            ""
        };

        let commit = RawCommit {
            id: "abc123def456".into(),
            short_id: "abc123".into(),
            summary: summary.into(),
            body: body.into(),
            author_name: "Test".into(),
            author_email: "test@test.com".into(),
            timestamp: 1704067200,
        };

        let parsed = parse_commit_fast(&commit);

        // Breaking should be true if either bang or footer indicates breaking
        if has_bang || has_footer {
            assert!(parsed.breaking, "Should detect breaking change");
        }
    }
}
