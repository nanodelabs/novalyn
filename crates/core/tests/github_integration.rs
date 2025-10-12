use novalyn_core::github::{GithubError, get_username_from_email, sync_release};
use novalyn_core::repository::{Provider, Repository};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path, query_param};

fn setup() {
    // Initialize crypto provider required by reqwest
    novalyn_core::init_crypto_provider();
}

#[tokio::test]
async fn test_get_username_from_email_success() {
    setup();
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search/users"))
        .and(query_param("q", "test@example.com+in:email"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": [{"login": "testuser", "email": "test@example.com"}]
            })),
        )
        .mount(&mock_server)
        .await;

    let result = get_username_from_email(
        "test@example.com",
        Some("test_token"),
        Some(&mock_server.uri()),
    )
    .await
    .unwrap();

    assert_eq!(result, Some("@testuser".into()));
}

#[tokio::test]
async fn test_get_username_from_email_not_found() {
    setup();
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/search/users"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "items": []
            })),
        )
        .mount(&mock_server)
        .await;

    let result = get_username_from_email(
        "notfound@example.com",
        Some("test_token"),
        Some(&mock_server.uri()),
    )
    .await
    .unwrap();

    assert_eq!(result, None);
}

#[tokio::test]
async fn test_get_username_from_email_no_token() {
    let result = get_username_from_email("test@example.com", None, None)
        .await
        .unwrap();

    assert_eq!(result, None);
}

#[tokio::test]
async fn test_sync_release_not_github() {
    let repo = novalyn_core::repository::Repository {
        provider: novalyn_core::repository::Provider::GitLab,
        host: "gitlab.com".into(),
        owner: "test".into(),
        name: "repo".into(),
        original: "https://gitlab.com/test/repo".into(),
    };

    let result = sync_release(&repo, Some("token"), "v1.0.0", "Release notes", None).await;

    assert!(matches!(result, Err(GithubError::NotGithub)));
}

#[tokio::test]
async fn test_sync_release_create_new() {
    setup();
    let mock_server = MockServer::start().await;

    // Mock GET to check if release exists (returns 404)
    Mock::given(method("GET"))
        .and(path("/repos/test/repo/releases/tags/v1.0.0"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    // Mock POST to create new release
    Mock::given(method("POST"))
        .and(path("/repos/test/repo/releases"))
        .respond_with(
            ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "html_url": "https://github.com/test/repo/releases/tag/v1.0.0",
                "tag_name": "v1.0.0"
            })),
        )
        .mount(&mock_server)
        .await;

    let repo = Repository {
        provider: Provider::GitHub,
        host: "github.com".into(),
        owner: "test".into(),
        name: "repo".into(),
        original: "https://github.com/test/repo".into(),
    };

    let result = sync_release(
        &repo,
        Some("test_token"),
        "v1.0.0",
        "Release notes",
        Some(&mock_server.uri()),
    )
    .await
    .unwrap();

    assert!(result.created);
    assert!(!result.updated);
    assert!(!result.skipped);
}

#[tokio::test]
async fn test_sync_release_update_existing() {
    setup();
    let mock_server = MockServer::start().await;

    // Mock GET to check if release exists (returns existing release)
    Mock::given(method("GET"))
        .and(path("/repos/test/repo/releases/tags/v1.0.0"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 123,
                "html_url": "https://github.com/test/repo/releases/tag/v1.0.0",
                "tag_name": "v1.0.0"
            })),
        )
        .mount(&mock_server)
        .await;

    // Mock PATCH to update existing release
    Mock::given(method("PATCH"))
        .and(path("/repos/test/repo/releases/123"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "html_url": "https://github.com/test/repo/releases/tag/v1.0.0",
                "tag_name": "v1.0.0"
            })),
        )
        .mount(&mock_server)
        .await;

    let repo = Repository {
        provider: Provider::GitHub,
        host: "github.com".into(),
        owner: "test".into(),
        name: "repo".into(),
        original: "https://github.com/test/repo".into(),
    };

    let result = sync_release(
        &repo,
        Some("test_token"),
        "v1.0.0",
        "Updated release notes",
        Some(&mock_server.uri()),
    )
    .await
    .unwrap();

    assert!(!result.created);
    assert!(result.updated);
    assert!(!result.skipped);
}
