use changelogen::github::sync_release;
use changelogen::repository::Repository;

#[tokio::test]
async fn github_sync_fallback_without_token() {
    // Use a GitHub-like repo struct
    let repo = Repository::parse("https://github.com/owner/repo.git").unwrap();
    let info = sync_release(&repo, None, "v0.1.0", "Body").await.unwrap();
    assert!(info.skipped);
    assert!(info.url.contains("/releases/tag/v0.1.0"));
}

#[tokio::test]
async fn github_sync_constructs_correct_manual_url() {
    // Test that manual URL is correctly constructed for various repo formats
    let repo = Repository::parse("https://github.com/test/repo.git").unwrap();
    let info = sync_release(&repo, None, "v1.2.3", "Release body")
        .await
        .unwrap();

    assert!(info.skipped, "should skip when no token");
    assert!(!info.created, "should not be marked as created");
    assert!(!info.updated, "should not be marked as updated");
    assert_eq!(info.tag, "v1.2.3");
    assert_eq!(info.url, "https://github.com/test/repo/releases/tag/v1.2.3");
}

#[tokio::test]
async fn github_sync_non_github_repo_error() {
    // Test that non-GitHub repos are rejected
    let repo = Repository::parse("https://gitlab.com/test/repo.git").unwrap();
    let result = sync_release(&repo, Some("token"), "v1.0.0", "Body").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.to_string(), "repository provider not GitHub");
}
