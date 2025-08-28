use serde::{Deserialize, Serialize};
use tracing::{debug, warn, instrument};

use crate::repository::Repository;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub tag: String,
    pub url: String,
    pub created: bool,
    pub updated: bool,
    pub skipped: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum GithubError {
    #[error("no repository information available for github sync")]
    NoRepo,
    #[error("repository provider not GitHub")]
    NotGithub,
    #[error("network error: {0}")]
    Network(String),
    #[error("unexpected response status {0}")]
    Status(u16),
}

/// Sync release with GitHub: get existing by tag, create or update.
/// Returns ReleaseInfo even on fallback path (manual URL) with skipped=true.
#[instrument(skip(token, body), fields(tag = %tag))]
pub async fn sync_release(
    repo: &Repository,
    token: Option<&str>,
    tag: &str,
    body: &str,
) -> Result<ReleaseInfo, GithubError> {
    if repo.provider != crate::repository::Provider::GitHub {
        return Err(GithubError::NotGithub);
    }
    let manual_url = format!(
        "https://github.com/{}/{}/releases/tag/{}",
        repo.owner, repo.name, tag
    );
    let Some(token) = token else {
        // No token -> fallback manual URL, mark skipped
        return Ok(ReleaseInfo {
            tag: tag.to_string(),
            url: manual_url,
            created: false,
            updated: false,
            skipped: true,
        });
    };
    let client = reqwest::Client::new();
    let api_base = format!(
        "https://api.github.com/repos/{}/{}/releases",
        repo.owner, repo.name
    );
    // 1. Try get by tag
    let get_url = format!("{}/tags/{}", api_base, tag);
    debug!("github_get_tag" = %get_url, "attempting fetch existing release");
    let existing = client
        .get(&get_url)
        .header("User-Agent", "changelogen-rs")
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| GithubError::Network(e.to_string()))?;
    if existing.status().as_u16() == 404 {
        // create new
        #[derive(Serialize)]
        struct CreateRelease<'a> {
            tag_name: &'a str,
            name: &'a str,
            body: &'a str,
            draft: bool,
            prerelease: bool,
        }
        let payload = CreateRelease {
            tag_name: tag,
            name: tag,
            body,
            draft: false,
            prerelease: false,
        };
        let resp = client
            .post(&api_base)
            .header("User-Agent", "changelogen-rs")
            .bearer_auth(token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| GithubError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(GithubError::Status(resp.status().as_u16()));
        }
        #[derive(Deserialize)]
        struct ReleaseResp {
            html_url: String,
        }
        let data: ReleaseResp = resp
            .json()
            .await
            .map_err(|e| GithubError::Network(e.to_string()))?;
        Ok(ReleaseInfo {
            tag: tag.to_string(),
            url: data.html_url,
            created: true,
            updated: false,
            skipped: false,
        })
    } else if existing.status().is_success() {
        #[derive(Deserialize)]
        struct ReleaseResp {
            id: u64,
            html_url: String,
        }
        let data: ReleaseResp = existing
            .json()
            .await
            .map_err(|e| GithubError::Network(e.to_string()))?;
        // update body if differs (simple unconditional patch)
        #[derive(Serialize)]
        struct UpdateRelease<'a> {
            body: &'a str,
        }
        let patch_url = format!("{}/{}", api_base, data.id);
        let resp = client
            .patch(&patch_url)
            .header("User-Agent", "changelogen-rs")
            .bearer_auth(token)
            .json(&UpdateRelease { body })
            .send()
            .await
            .map_err(|e| GithubError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            warn!(status = %resp.status(), "github update release failed");
        }
        Ok(ReleaseInfo {
            tag: tag.to_string(),
            url: data.html_url,
            created: false,
            updated: true,
            skipped: false,
        })
    } else {
        warn!(status = %existing.status(), "github get release unexpected status");
        // fallback manual
        Ok(ReleaseInfo {
            tag: tag.to_string(),
            url: manual_url,
            created: false,
            updated: false,
            skipped: true,
        })
    }
}
