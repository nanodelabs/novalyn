//! Property-based tests for configuration handling
//!
//! Tests that configuration merging, validation, and type resolution work correctly.

use novalyn_core::config::{SemverImpact, TypeConfigResolved, default_types};
use proptest::prelude::*;

// Strategy for generating valid commit type keys
fn valid_type_key() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z]{2,15}").unwrap()
}

// Strategy for generating emoji strings
fn emoji_string() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "✨".to_string(),
        "🐞".to_string(),
        "⚡️".to_string(),
        "📚".to_string(),
        "🛠".to_string(),
        "🎨".to_string(),
        "🧪".to_string(),
        "📦".to_string(),
        "👷".to_string(),
        "🧹".to_string(),
        "⏪".to_string(),
    ])
}

// Strategy for generating title strings
fn title_string() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Z][a-zA-Z ]{2,30}").unwrap()
}

// Strategy for generating SemverImpact
fn semver_impact() -> impl Strategy<Value = SemverImpact> {
    prop::sample::select(vec![
        SemverImpact::Major,
        SemverImpact::Minor,
        SemverImpact::Patch,
        SemverImpact::None,
    ])
}

proptest! {
    /// Test that TypeConfigResolved can be created with various valid inputs
    #[test]
    fn type_config_creation(
        key in valid_type_key(),
        title in title_string(),
        emoji in emoji_string(),
        semver in semver_impact(),
        enabled in proptest::bool::ANY,
    ) {
        let config = TypeConfigResolved {
            key: key.clone().into(),
            title: title.into(),
            emoji: emoji.into(),
            semver,
            enabled,
        };

        assert_eq!(config.key.as_str(), key);
        assert_eq!(config.enabled, enabled);
        assert_eq!(config.semver, semver);
    }

    /// Test that SemverImpact string parsing is case-sensitive
    #[test]
    fn semver_impact_string_parsing(
        valid in prop::sample::select(vec!["major", "minor", "patch", "none"])
    ) {
        let parsed = SemverImpact::parse(valid);
        assert!(parsed.is_some(), "Should parse valid semver impact: {}", valid);
    }

    /// Test that invalid SemverImpact strings return None
    #[test]
    fn semver_impact_invalid_strings(
        invalid in prop::string::string_regex("[A-Z]{1,10}").unwrap()
    ) {
        // Uppercase versions should not parse
        let parsed = SemverImpact::parse(&invalid);
        assert!(parsed.is_none(), "Should not parse: {}", invalid);
    }
}

// Regular unit tests for default configuration
#[cfg(test)]
mod default_config_tests {
    use super::*;

    #[test]
    fn default_types_order_consistent() {
        let types = default_types();

        // Should always have the standard types in order
        assert!(types.len() >= 11); // At least the standard 11 types

        // First few should be feat, fix, perf in that order
        assert_eq!(types[0].key.as_str(), "feat");
        assert_eq!(types[1].key.as_str(), "fix");
        assert_eq!(types[2].key.as_str(), "perf");
    }

    #[test]
    fn default_types_have_all_fields() {
        let types = default_types();

        for t in &types {
            assert!(!t.key.is_empty(), "Type key should not be empty");
            assert!(!t.title.is_empty(), "Type title should not be empty");
            assert!(!t.emoji.is_empty(), "Type emoji should not be empty");
            assert!(t.enabled, "Default types should be enabled");
        }
    }

    #[test]
    fn default_types_keys_unique() {
        let types = default_types();
        let mut seen = std::collections::HashSet::new();

        for t in &types {
            assert!(seen.insert(t.key.clone()), "Duplicate key: {}", t.key);
        }
    }

    #[test]
    fn default_types_semver_sensible() {
        let types = default_types();

        // feat should always be Minor
        let feat = types.iter().find(|t| t.key.as_str() == "feat").unwrap();
        assert_eq!(feat.semver, SemverImpact::Minor);

        // fix should always be Patch
        let fix = types.iter().find(|t| t.key.as_str() == "fix").unwrap();
        assert_eq!(fix.semver, SemverImpact::Patch);

        // docs should always be None
        let docs = types.iter().find(|t| t.key.as_str() == "docs").unwrap();
        assert_eq!(docs.semver, SemverImpact::None);
    }

    #[test]
    fn default_types_naming_conventions() {
        let types = default_types();

        for t in &types {
            // Keys should be lowercase alphabetic
            assert!(
                t.key.chars().all(|c| c.is_ascii_lowercase()),
                "Key should be lowercase: {}",
                t.key
            );

            // Titles should start with uppercase
            assert!(
                t.title.chars().next().unwrap().is_ascii_uppercase(),
                "Title should start with uppercase: {}",
                t.title
            );
        }
    }
}

// Additional unit tests for specific functionality
#[cfg(test)]
mod config_specifics {
    use super::*;

    #[test]
    fn test_semver_impact_ordering() {
        // Major > Minor > Patch > None in severity
        let impacts = [
            SemverImpact::None,
            SemverImpact::Patch,
            SemverImpact::Minor,
            SemverImpact::Major,
        ];

        // Just verify they are all distinct
        for i in 0..impacts.len() {
            for j in 0..impacts.len() {
                if i == j {
                    assert_eq!(impacts[i], impacts[j]);
                } else {
                    assert_ne!(impacts[i], impacts[j]);
                }
            }
        }
    }

    #[test]
    fn test_default_types_completeness() {
        let types = default_types();
        let expected = [
            "feat", "fix", "perf", "docs", "refactor", "style", "test", "build", "ci", "chore",
            "revert",
        ];

        for expected_key in &expected {
            assert!(
                types.iter().any(|t| t.key.as_str() == *expected_key),
                "Missing expected type: {}",
                expected_key
            );
        }
    }

    #[test]
    fn test_semver_impact_from_str() {
        assert_eq!(SemverImpact::parse("major"), Some(SemverImpact::Major));
        assert_eq!(SemverImpact::parse("minor"), Some(SemverImpact::Minor));
        assert_eq!(SemverImpact::parse("patch"), Some(SemverImpact::Patch));
        assert_eq!(SemverImpact::parse("none"), Some(SemverImpact::None));

        // Invalid inputs
        assert_eq!(SemverImpact::parse("Major"), None);
        assert_eq!(SemverImpact::parse("MAJOR"), None);
        assert_eq!(SemverImpact::parse(""), None);
        assert_eq!(SemverImpact::parse("invalid"), None);
    }

    #[test]
    fn test_type_config_enabled_flag() {
        let enabled = TypeConfigResolved {
            key: "test".into(),
            title: "Test".into(),
            emoji: "🧪".into(),
            semver: SemverImpact::None,
            enabled: true,
        };
        assert!(enabled.enabled);

        let disabled = TypeConfigResolved {
            key: "test".into(),
            title: "Test".into(),
            emoji: "🧪".into(),
            semver: SemverImpact::None,
            enabled: false,
        };
        assert!(!disabled.enabled);
    }
}
