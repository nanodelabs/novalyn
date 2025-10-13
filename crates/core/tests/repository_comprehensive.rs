use novalyn_core::repository::{
    Provider, ReferenceKind, Repository, format_compare_changes, format_reference,
};

/// Create a Repository instance for the given provider, host, owner, and name.
fn make_repo(provider: Provider, host: &str, owner: &str, name: &str) -> Repository {
    Repository {
        provider,
        host: host.into(),
        owner: owner.into(),
        name: name.into(),
        original: format!("https://{}/{}/{}", host, owner, name).into(),
    }
}

#[test]
fn test_provider_display() {
    assert_eq!(Provider::GitHub.to_string(), "GitHub");
    assert_eq!(Provider::GitLab.to_string(), "GitLab");
    assert_eq!(Provider::Bitbucket.to_string(), "Bitbucket");
    assert_eq!(Provider::Other.to_string(), "Other");
}

#[test]
fn test_format_reference_issue_github() {
    let repo = make_repo(Provider::GitHub, "github.com", "user", "project");

    let result = format_reference(Some(&repo), ReferenceKind::Issue, "#123");
    assert_eq!(
        result.as_str(),
        "[#123](https://github.com/user/project/issues/123)"
    );
}

#[test]
fn test_format_reference_issue_gitlab() {
    let repo = make_repo(Provider::GitLab, "gitlab.com", "user", "project");

    let result = format_reference(Some(&repo), ReferenceKind::Issue, "#456");
    assert_eq!(
        result.as_str(),
        "[#456](https://gitlab.com/user/project/issues/456)"
    );
}

#[test]
fn test_format_reference_pr_github() {
    let repo = make_repo(Provider::GitHub, "github.com", "user", "project");

    let result = format_reference(Some(&repo), ReferenceKind::PullRequest, "#789");
    assert_eq!(
        result.as_str(),
        "[#789](https://github.com/user/project/pull/789)"
    );
}

#[test]
fn test_format_reference_commit_github() {
    let repo = make_repo(Provider::GitHub, "github.com", "user", "project");

    let result = format_reference(Some(&repo), ReferenceKind::Hash, "abc123");
    assert_eq!(
        result.as_str(),
        "[abc123](https://github.com/user/project/commit/abc123)"
    );
}

#[test]
fn test_format_reference_without_repo() {
    let result = format_reference(None, ReferenceKind::Issue, "#123");
    assert_eq!(result.as_str(), "#123");
}

#[test]
fn test_format_compare_changes_github() {
    let repo = make_repo(Provider::GitHub, "github.com", "user", "project");

    let result = format_compare_changes(None, "v1.0.0", "v1.1.0", Some(&repo));
    assert!(result.is_some());
    let link = result.unwrap();
    assert!(link.contains("compare/v1.0.0...v1.1.0"));
    assert!(link.contains("github.com/user/project"));
}

#[test]
fn test_format_compare_changes_gitlab() {
    let repo = make_repo(Provider::GitLab, "gitlab.com", "user", "project");

    let result = format_compare_changes(None, "v1.0.0", "v1.1.0", Some(&repo));
    assert!(result.is_some());
    let link = result.unwrap();
    assert!(link.contains("compare/v1.0.0...v1.1.0"));
    assert!(link.contains("gitlab.com/user/project"));
}

#[test]
fn test_format_compare_changes_without_repo() {
    let result = format_compare_changes(None, "v1.0.0", "v1.1.0", None);
    assert!(result.is_none());
}

#[test]
fn test_format_compare_changes_with_base_url() {
    let repo = make_repo(Provider::GitHub, "github.com", "user", "project");

    let result = format_compare_changes(
        Some("https://custom.github.com"),
        "v1.0.0",
        "v1.1.0",
        Some(&repo),
    );
    assert!(result.is_some());
    let link = result.unwrap();
    assert!(link.contains("custom.github.com"));
}

#[test]
fn test_repository_equality() {
    let repo1 = make_repo(Provider::GitHub, "github.com", "user", "project");
    let repo2 = make_repo(Provider::GitHub, "github.com", "user", "project");
    let repo3 = make_repo(Provider::GitLab, "gitlab.com", "user", "project");

    assert_eq!(repo1, repo2);
    assert_ne!(repo1, repo3);
}

#[test]
fn test_issue_url() {
    let repo = make_repo(Provider::GitHub, "github.com", "user", "project");

    let url = repo.issue_url(42);
    assert_eq!(url.as_str(), "https://github.com/user/project/issues/42");
}

#[test]
fn test_pr_url() {
    let repo = make_repo(Provider::GitHub, "github.com", "user", "project");

    let url = repo.pr_url(99);
    assert_eq!(url.as_str(), "https://github.com/user/project/pull/99");
}

#[test]
fn test_commit_url() {
    let repo = make_repo(Provider::GitHub, "github.com", "user", "project");

    let url = repo.commit_url("deadbeef");
    assert_eq!(
        url.as_str(),
        "https://github.com/user/project/commit/deadbeef"
    );
}

#[test]
fn test_repository_parse_ssh() {
    let repo = Repository::parse("git@github.com:user/project.git");
    assert!(repo.is_some());
    let repo = repo.unwrap();
    assert_eq!(repo.host.as_str(), "github.com");
    assert_eq!(repo.owner.as_str(), "user");
    assert_eq!(repo.name.as_str(), "project");
    assert_eq!(repo.provider, Provider::GitHub);
}

#[test]
fn test_repository_parse_https() {
    let repo = Repository::parse("https://gitlab.com/user/project.git");
    assert!(repo.is_some());
    let repo = repo.unwrap();
    assert_eq!(repo.host.as_str(), "gitlab.com");
    assert_eq!(repo.owner.as_str(), "user");
    assert_eq!(repo.name.as_str(), "project");
    assert_eq!(repo.provider, Provider::GitLab);
}

#[test]
fn test_repository_parse_invalid() {
    let repo = Repository::parse("invalid-url");
    assert!(repo.is_none());
}
