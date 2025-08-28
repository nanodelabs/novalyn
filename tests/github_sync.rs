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
