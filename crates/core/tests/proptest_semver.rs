//! Property-based tests for semver bump logic
//!
//! Tests invariants and properties of version bump escalation.

use novalyn_core::parse::BumpKind;
use proptest::prelude::*;

// Strategy for generating BumpKind values
prop_compose! {
    fn arb_bump_kind()(val in 0..4u8) -> BumpKind {
        match val {
            0 => BumpKind::Major,
            1 => BumpKind::Minor,
            2 => BumpKind::Patch,
            _ => BumpKind::None,
        }
    }
}

proptest! {
    fn escalate_is_commutative(a in arb_bump_kind(), b in arb_bump_kind()) {
        prop_assert_eq!(a.escalate(b), b.escalate(a));
    }

    fn escalate_is_associative(
        a in arb_bump_kind(),
        b in arb_bump_kind(),
        c in arb_bump_kind()
    ) {
        let result1 = a.escalate(b).escalate(c);
        let result2 = a.escalate(b.escalate(c));
        prop_assert_eq!(result1, result2);
    }

    fn none_is_identity(bump in arb_bump_kind()) {
        prop_assert_eq!(bump.escalate(BumpKind::None), bump);
        prop_assert_eq!(BumpKind::None.escalate(bump), bump);
    }

    fn major_dominates(bump in arb_bump_kind()) {
        prop_assert_eq!(BumpKind::Major.escalate(bump), BumpKind::Major);
        prop_assert_eq!(bump.escalate(BumpKind::Major), BumpKind::Major);
    }

    fn patch_dominates_none(_seed in 0..1u8) {
        prop_assert_eq!(BumpKind::Patch.escalate(BumpKind::None), BumpKind::Patch);
        prop_assert_eq!(BumpKind::None.escalate(BumpKind::Patch), BumpKind::Patch);
    }

    fn escalate_sequence(bumps in prop::collection::vec(arb_bump_kind(), 1..20)) {
        // Starting with None
        let result = bumps.iter().fold(BumpKind::None, |acc, &b| acc.escalate(b));

        // Result should be highest bump in the sequence
        let has_major = bumps.contains(&BumpKind::Major);
        let has_minor = bumps.contains(&BumpKind::Minor);
        let has_patch = bumps.contains(&BumpKind::Patch);

        if has_major {
            prop_assert_eq!(result, BumpKind::Major);
        } else if has_minor {
            prop_assert_eq!(result, BumpKind::Minor);
        } else if has_patch {
            prop_assert_eq!(result, BumpKind::Patch);
        } else {
            prop_assert_eq!(result, BumpKind::None);
        }
    }

    fn escalate_is_idempotent(bump in arb_bump_kind()) {
        prop_assert_eq!(bump.escalate(bump), bump);
    }

    fn multiple_escalations_order_independent(
        bumps in prop::collection::vec(arb_bump_kind(), 2..10)
    ) {
        // Calculate result in original order
        let forward = bumps.iter().fold(BumpKind::None, |acc, &b| acc.escalate(b));

        // Calculate result in reverse order
        let backward = bumps.iter().rev().fold(BumpKind::None, |acc, &b| acc.escalate(b));

        prop_assert_eq!(forward, backward);
    }
}

// Additional unit tests for specific scenarios
#[cfg(test)]
mod specific_scenarios {
    use super::*;

    #[test]
    fn test_typical_release_progression() {
        // Typical progression: patches accumulate, then minor, then major
        let sequence = [
            BumpKind::Patch,
            BumpKind::Patch,
            BumpKind::Minor,
            BumpKind::Patch,
        ];

        let result = sequence
            .iter()
            .fold(BumpKind::None, |acc, &b| acc.escalate(b));
        assert_eq!(result, BumpKind::Minor);
    }

    #[test]
    fn test_breaking_change_override() {
        // Even with many patches and minors, a major bump wins
        let sequence = [
            BumpKind::Patch,
            BumpKind::Minor,
            BumpKind::Patch,
            BumpKind::Major,
            BumpKind::Patch,
        ];

        let result = sequence
            .iter()
            .fold(BumpKind::None, |acc, &b| acc.escalate(b));
        assert_eq!(result, BumpKind::Major);
    }

    #[test]
    fn test_no_changes() {
        // All None should result in None
        let sequence = [BumpKind::None; 10];
        let result = sequence
            .iter()
            .fold(BumpKind::None, |acc, &b| acc.escalate(b));
        assert_eq!(result, BumpKind::None);
    }

    #[test]
    fn test_single_bump() {
        // Single bump of each type
        assert_eq!(BumpKind::None.escalate(BumpKind::Major), BumpKind::Major);
        assert_eq!(BumpKind::None.escalate(BumpKind::Minor), BumpKind::Minor);
        assert_eq!(BumpKind::None.escalate(BumpKind::Patch), BumpKind::Patch);
    }

    #[test]
    fn test_all_combinations() {
        let all_bumps = [
            BumpKind::Major,
            BumpKind::Minor,
            BumpKind::Patch,
            BumpKind::None,
        ];

        for &a in &all_bumps {
            for &b in &all_bumps {
                // Just verify it doesn't panic and returns a valid BumpKind
                let result = a.escalate(b);
                assert!(matches!(
                    result,
                    BumpKind::Major | BumpKind::Minor | BumpKind::Patch | BumpKind::None
                ));
            }
        }
    }

    #[test]
    fn test_minor_dominates_patch() {
        assert_eq!(BumpKind::Minor.escalate(BumpKind::Patch), BumpKind::Minor);
        assert_eq!(BumpKind::Patch.escalate(BumpKind::Minor), BumpKind::Minor);
    }

    #[test]
    fn test_minor_dominates_none() {
        assert_eq!(BumpKind::Minor.escalate(BumpKind::None), BumpKind::Minor);
        assert_eq!(BumpKind::None.escalate(BumpKind::Minor), BumpKind::Minor);
    }
}
