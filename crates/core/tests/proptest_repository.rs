//! Property-based tests for repository URL parsing
//!
//! Tests that repository parsing handles various URL formats correctly.

use novalyn_core::repository::{Provider, Repository};
use proptest::prelude::*;

// Strategy for valid repository owner/org names
fn valid_owner() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9][a-zA-Z0-9-]{0,38}").unwrap()
}

// Strategy for valid repository names
fn valid_repo_name() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9._-]{1,100}").unwrap()
}

// Strategy for valid hostnames
fn valid_hostname() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "github.com".to_string(),
        "gitlab.com".to_string(),
        "bitbucket.org".to_string(),
        "git.example.com".to_string(),
        "gitlab.example.org".to_string(),
    ])
}

proptest! {
    /// Test that parsing never panics on arbitrary input
    #[test]
    fn parse_never_panics(url in "\\PC*") {
        let _ = Repository::parse(&url);
    }

    /// Test that well-formed HTTPS URLs are parsed correctly
    #[test]
    fn parse_https_urls(
        host in valid_hostname(),
        owner in valid_owner(),
        name in valid_repo_name(),
        has_git_suffix in proptest::bool::ANY,
    ) {
        let suffix = if has_git_suffix { ".git" } else { "" };
        let url = format!("https://{}/{}/{}{}", host, owner, name, suffix);

        let repo = Repository::parse(&url);
        assert!(repo.is_some(), "Failed to parse valid HTTPS URL: {}", url);

        let repo = repo.unwrap();
        assert_eq!(repo.host.as_str(), host);
        assert_eq!(repo.owner.as_str(), owner);
        assert_eq!(repo.name.as_str(), name);

        // Verify provider detection
        match host.as_str() {
            "github.com" => assert_eq!(repo.provider, Provider::GitHub),
            "gitlab.com" => assert_eq!(repo.provider, Provider::GitLab),
            "bitbucket.org" => assert_eq!(repo.provider, Provider::Bitbucket),
            _ => assert_eq!(repo.provider, Provider::Other),
        }
    }

    /// Test that well-formed SSH URLs are parsed correctly
    #[test]
    fn parse_ssh_urls(
        host in valid_hostname(),
        owner in valid_owner(),
        name in valid_repo_name(),
        has_git_suffix in proptest::bool::ANY,
    ) {
        let suffix = if has_git_suffix { ".git" } else { "" };
        let url = format!("git@{}:{}/{}{}", host, owner, name, suffix);

        let repo = Repository::parse(&url);
        assert!(repo.is_some(), "Failed to parse valid SSH URL: {}", url);

        let repo = repo.unwrap();
        assert_eq!(repo.host.as_str(), host);
        assert_eq!(repo.owner.as_str(), owner);
        assert_eq!(repo.name.as_str(), name);
    }

    /// Test that SSH alternative format is parsed correctly
    #[test]
    fn parse_ssh_alternative(
        host in valid_hostname(),
        owner in valid_owner(),
        name in valid_repo_name(),
        has_git_suffix in proptest::bool::ANY,
    ) {
        let suffix = if has_git_suffix { ".git" } else { "" };
        let url = format!("ssh://git@{}/{}/{}{}", host, owner, name, suffix);

        let repo = Repository::parse(&url);
        assert!(repo.is_some(), "Failed to parse valid SSH alternative URL: {}", url);

        let repo = repo.unwrap();
        assert_eq!(repo.host.as_str(), host);
        assert_eq!(repo.owner.as_str(), owner);
        assert_eq!(repo.name.as_str(), name);
    }

    /// Test that HTTP URLs are parsed (rare but valid)
    #[test]
    fn parse_http_urls(
        host in valid_hostname(),
        owner in valid_owner(),
        name in valid_repo_name(),
    ) {
        let url = format!("http://{}/{}/{}", host, owner, name);

        let repo = Repository::parse(&url);
        assert!(repo.is_some(), "Failed to parse valid HTTP URL: {}", url);

        let repo = repo.unwrap();
        assert_eq!(repo.host.as_str(), host);
        assert_eq!(repo.owner.as_str(), owner);
        assert_eq!(repo.name.as_str(), name);
    }

    /// Test that invalid URLs return None
    #[test]
    fn invalid_urls_return_none(
        invalid in prop::sample::select(vec![
            "",
            "not-a-url",
            "https://",
            "git@",
            "https://host.com/",
            "https://host.com/owner",
            "git@host.com:",
            "git@host.com:owner",
            // URL with extra path segments (not currently supported)
            "https://gitlab.com/owner/group/project",
        ])
    ) {
        let result = Repository::parse(invalid);
        assert!(result.is_none(), "Should return None for invalid URL: {}", invalid);
    }

    /// Test that trailing slashes are handled
    #[test]
    fn handles_trailing_slashes(
        host in valid_hostname(),
        owner in valid_owner(),
        name in valid_repo_name(),
        trailing_slashes in 0..5usize,
    ) {
        let slashes = "/".repeat(trailing_slashes);
        let url = format!("https://{}/{}/{}{}", host, owner, name, slashes);

        let repo = Repository::parse(&url);
        assert!(repo.is_some(), "Failed to parse URL with trailing slashes: {}", url);

        let repo = repo.unwrap();
        assert_eq!(repo.name.as_str(), name);
    }

    /// Test that .git suffix is properly stripped
    #[test]
    fn strips_git_suffix(
        host in valid_hostname(),
        owner in valid_owner(),
        name in valid_repo_name(),
    ) {
        let url_with_git = format!("https://{}/{}/{}.git", host, owner, name);
        let url_without_git = format!("https://{}/{}/{}", host, owner, name);

        let repo1 = Repository::parse(&url_with_git).unwrap();
        let repo2 = Repository::parse(&url_without_git).unwrap();

        // Both should parse to the same name (without .git)
        assert_eq!(repo1.name, repo2.name);
        assert_eq!(repo1.name.as_str(), name);
    }

    /// Test Repository equality
    #[test]
    fn repository_equality(
        host in valid_hostname(),
        owner in valid_owner(),
        name in valid_repo_name(),
    ) {
        let url1 = format!("https://{}/{}/{}", host, owner, name);
        let url2 = format!("git@{}:{}/{}.git", host, owner, name);

        let repo1 = Repository::parse(&url1).unwrap();
        let repo2 = Repository::parse(&url2).unwrap();

        // Different URL formats should parse to equal repositories
        // (except original field which preserves the input)
        assert_eq!(repo1.host, repo2.host);
        assert_eq!(repo1.owner, repo2.owner);
        assert_eq!(repo1.name, repo2.name);
        assert_eq!(repo1.provider, repo2.provider);
    }

    /// Test that URL formatting methods work correctly
    #[test]
    fn url_formatting_consistency(
        host in prop::sample::select(vec!["github.com", "gitlab.com", "bitbucket.org"]),
        owner in valid_owner(),
        name in valid_repo_name(),
    ) {
        use novalyn_core::repository::{format_reference, ReferenceKind};

        let url = format!("https://{}/{}/{}", host, owner, name);
        let repo = Repository::parse(&url).unwrap();

        // Test that formatted URLs contain the repository components
        let issue_url = format_reference(Some(&repo), ReferenceKind::Issue, "42");
        assert!(issue_url.contains(&owner));
        assert!(issue_url.contains(&name));
        assert!(issue_url.contains("42"));

        let commit_url = format_reference(Some(&repo), ReferenceKind::Hash, "abc123");
        assert!(commit_url.contains(&owner));
        assert!(commit_url.contains(&name));
        assert!(commit_url.contains("abc123"));
    }
}

// Additional unit tests for edge cases
#[cfg(test)]
mod edge_cases {
    use super::*;

    #[test]
    fn test_case_preservation() {
        let url = "https://github.com/MyOwner/MyRepo";
        let repo = Repository::parse(url).unwrap();
        assert_eq!(repo.owner.as_str(), "MyOwner");
        assert_eq!(repo.name.as_str(), "MyRepo");
    }

    #[test]
    fn test_special_characters_in_names() {
        // Dots, underscores, dashes are common in repo names
        let url = "https://github.com/owner/my-repo.name_test";
        let repo = Repository::parse(url).unwrap();
        assert_eq!(repo.name.as_str(), "my-repo.name_test");
    }

    #[test]
    fn test_numeric_names() {
        let url = "https://github.com/123owner/456repo";
        let repo = Repository::parse(url);
        // This might be valid or invalid depending on platform rules
        // At minimum, it shouldn't panic
        let _ = repo;
    }
}
