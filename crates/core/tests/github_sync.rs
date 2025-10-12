use novalyn_core::github::sync_release;
use novalyn_core::repository::Repository;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Helper module for wiremock tests that require rustls initialization
mod wiremock_helpers {
    use std::sync::Once;
    static INIT: Once = Once::new();

    pub fn setup() {
        INIT.call_once(|| {
            novalyn_core::init_crypto_provider();
        });
    }
}

#[tokio::test]
async fn github_sync_fallback_without_token() {
    // Use a GitHub-like repo struct
    let repo = Repository::parse("https://github.com/owner/repo.git").unwrap();
    let info = sync_release(&repo, None, "v0.1.0", "Body", None)
        .await
        .unwrap();
    assert!(info.skipped);
    assert!(info.url.contains("/releases/tag/v0.1.0"));
}

#[tokio::test]
async fn github_sync_constructs_correct_manual_url() {
    // Test that manual URL is correctly constructed for various repo formats
    let repo = Repository::parse("https://github.com/test/repo.git").unwrap();
    let info = sync_release(&repo, None, "v1.2.3", "Release body", None)
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
    let result = sync_release(&repo, Some("token"), "v1.0.0", "Body", None).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.to_string(), "repository provider not GitHub");
}

#[tokio::test]
async fn github_sync_create_release_with_wiremock() {
    wiremock_helpers::setup();
    let mock_server = MockServer::start().await;

    // Mock GET request returning 404 (release doesn't exist)
    Mock::given(method("GET"))
        .and(path("/repos/test/repo/releases/tags/v1.0.0"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    // Mock POST request to create release
    Mock::given(method("POST"))
        .and(path("/repos/test/repo/releases"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "html_url": "https://github.com/test/repo/releases/tag/v1.0.0"
        })))
        .mount(&mock_server)
        .await;

    let repo = Repository::parse("https://github.com/test/repo.git").unwrap();
    let info = sync_release(
        &repo,
        Some("test-token"),
        "v1.0.0",
        "Release body",
        Some(&mock_server.uri()),
    )
    .await
    .unwrap();

    assert!(info.created);
    assert!(!info.updated);
    assert!(!info.skipped);
    assert!(info.url.contains("releases/tag/v1.0.0"));
}

#[tokio::test]
async fn github_sync_update_existing_release_with_wiremock() {
    wiremock_helpers::setup();
    let mock_server = MockServer::start().await;

    // Mock GET request returning existing release
    Mock::given(method("GET"))
        .and(path("/repos/test/repo/releases/tags/v1.0.0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 12345,
            "html_url": "https://github.com/test/repo/releases/tag/v1.0.0"
        })))
        .mount(&mock_server)
        .await;

    // Mock PATCH request to update release
    Mock::given(method("PATCH"))
        .and(path("/repos/test/repo/releases/12345"))
        .and(header("authorization", "Bearer test-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let repo = Repository::parse("https://github.com/test/repo.git").unwrap();
    let info = sync_release(
        &repo,
        Some("test-token"),
        "v1.0.0",
        "Updated body",
        Some(&mock_server.uri()),
    )
    .await
    .unwrap();

    assert!(!info.created);
    assert!(info.updated);
    assert!(!info.skipped);
}
