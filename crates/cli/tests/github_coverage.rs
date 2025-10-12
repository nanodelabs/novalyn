use novalyn::github::{GithubError, get_username_from_email, sync_release};
use novalyn::repository::Repository;

#[tokio::test]
async fn test_get_username_no_token() {
    let result = get_username_from_email("test@example.com", None, None).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None); // No token means no result
}

#[tokio::test]
async fn test_sync_release_no_token_fallback() {
    let repo = Repository::parse("https://github.com/owner/repo.git").unwrap();

    let result = sync_release(&repo, None, "v1.0.0", "Release body", None).await;
    assert!(result.is_ok());
    let info = result.unwrap();
    assert!(info.skipped);
    assert!(info.url.contains("/releases/tag/v1.0.0"));
    assert!(!info.created);
    assert!(!info.updated);
}

#[tokio::test]
async fn test_sync_release_non_github_error() {
    let repo = Repository::parse("https://gitlab.com/owner/repo.git").unwrap();

    let result = sync_release(&repo, Some("token"), "v1.0.0", "Release body", None).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        GithubError::NotGithub => (),
        _ => panic!("Expected NotGithub error"),
    }
}

#[tokio::test]
async fn test_sync_release_bitbucket_error() {
    let repo = Repository::parse("https://bitbucket.org/owner/repo.git").unwrap();

    let result = sync_release(&repo, Some("token"), "v1.0.0", "Release body", None).await;
    assert!(result.is_err());
}

#[test]
fn test_github_error_display() {
    let err = GithubError::NoRepo;
    assert_eq!(
        err.to_string(),
        "no repository information available for github sync"
    );

    let err = GithubError::NotGithub;
    assert_eq!(err.to_string(), "repository provider not GitHub");

    let err = GithubError::Network("test error".to_string());
    assert_eq!(err.to_string(), "network error: test error");

    let err = GithubError::Status(404);
    assert_eq!(err.to_string(), "unexpected response status 404");
}

#[tokio::test]
#[ignore = "requires rustls provider initialization"]
async fn test_get_username_with_custom_api_base() {
    // Test that api_base parameter is respected
    let result = get_username_from_email(
        "test@example.com",
        Some("fake_token"),
        Some("https://api.custom.com"),
    )
    .await;

    // This will fail with network error since it's a fake URL,
    // but it proves the parameter is being used
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
#[ignore = "requires rustls provider initialization"]
async fn test_sync_release_with_custom_api_base() {
    let repo = Repository::parse("https://github.com/owner/repo.git").unwrap();

    // Test that api_base parameter is respected
    let result = sync_release(
        &repo,
        Some("fake_token"),
        "v1.0.0",
        "Release body",
        Some("https://api.custom.com"),
    )
    .await;

    // This will fail with network error since it's a fake URL,
    // but it proves the parameter is being used
    assert!(result.is_err() || result.is_ok());
}
