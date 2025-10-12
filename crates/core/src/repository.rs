use ecow::EcoString;
use std::fmt;

/// Repository hosting provider.
///
/// Determines URL formats for issues, PRs, commits, and compare views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    /// GitHub
    GitHub,
    /// GitLab
    GitLab,
    /// Bitbucket
    Bitbucket,
    /// Other or unknown provider
    Other,
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::GitHub => write!(f, "GitHub"),
            Provider::GitLab => write!(f, "GitLab"),
            Provider::Bitbucket => write!(f, "Bitbucket"),
            Provider::Other => write!(f, "Other"),
        }
    }
}

/// Git repository information parsed from remote URL.
///
/// Contains provider, host, owner, and project name for URL formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    pub host: EcoString,
    pub owner: EcoString,
    pub name: EcoString,
    pub provider: Provider,
    /// Original remote URL
    pub original: EcoString,
}

/// Type of git reference for URL formatting.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReferenceKind {
    PullRequest,
    Issue,
    Hash,
}

impl Repository {
    pub fn parse(remote: &str) -> Option<Self> {
        // Try SSH: git@host:owner/name(.git)
        if let Some(rest) = remote.strip_prefix("git@") {
            let mut parts = rest.splitn(2, ':');
            let host = parts.next()?.into();
            let path = parts.next()?;
            return Self::from_host_path(host, path, remote);
        }
        // SSH alternative: ssh://git@host/owner/name(.git)
        if let Some(stripped) = remote.strip_prefix("ssh://git@")
            && let Some((host, path)) = stripped.split_once('/')
        {
            return Self::from_host_path(host.into(), path, remote);
        }
        // HTTPS: https://host/owner/name(.git)
        if let Some(stripped) = remote.strip_prefix("https://")
            && let Some((host, path)) = stripped.split_once('/')
        {
            return Self::from_host_path(host.into(), path, remote);
        }
        // HTTP (rare)
        if let Some(stripped) = remote.strip_prefix("http://")
            && let Some((host, path)) = stripped.split_once('/')
        {
            return Self::from_host_path(host.into(), path, remote);
        }
        None
    }

    fn from_host_path(host: EcoString, path: &str, original: &str) -> Option<Self> {
        let path = path.trim_end_matches('/').trim_end_matches(".git");
        let mut segs = path.split('/');
        let owner = segs.next()?.into();
        let name = segs.next()?.into();
        if segs.next().is_some() {
            return None;
        } // extra segments unsupported (subgroups future)
        let provider = match host.as_str() {
            "github.com" => Provider::GitHub,
            "gitlab.com" => Provider::GitLab,
            "bitbucket.org" => Provider::Bitbucket,
            _ => Provider::Other,
        };
        Some(Self {
            host,
            owner,
            name,
            provider,
            original: original.into(),
        })
    }

    pub fn commit_url(&self, sha: &str) -> EcoString {
        match self.provider {
            Provider::GitHub | Provider::GitLab => format!(
                "https://{}/{}/{}/commit/{}",
                self.host, self.owner, self.name, sha
            )
            .into(),
            Provider::Bitbucket => format!(
                "https://{}/{}/{}/commits/{}",
                self.host, self.owner, self.name, sha
            )
            .into(),
            Provider::Other => EcoString::new(),
        }
    }
    pub fn tag_url(&self, tag: &str) -> EcoString {
        match self.provider {
            Provider::GitHub | Provider::GitLab => format!(
                "https://{}/{}/{}/releases/tag/{}",
                self.host, self.owner, self.name, tag
            )
            .into(),
            Provider::Bitbucket => format!(
                "https://{}/{}/{}/src/{}",
                self.host, self.owner, self.name, tag
            )
            .into(),
            Provider::Other => EcoString::new(),
        }
    }
    pub fn issue_url(&self, num: u64) -> EcoString {
        match self.provider {
            Provider::GitHub | Provider::GitLab => format!(
                "https://{}/{}/{}/issues/{}",
                self.host, self.owner, self.name, num
            )
            .into(),
            Provider::Bitbucket => format!(
                "https://{}/{}/{}/issues/{}",
                self.host, self.owner, self.name, num
            )
            .into(),
            Provider::Other => EcoString::new(),
        }
    }
    pub fn pr_url(&self, num: u64) -> EcoString {
        match self.provider {
            Provider::GitHub => format!(
                "https://{}/{}/{}/pull/{}",
                self.host, self.owner, self.name, num
            )
            .into(),
            Provider::GitLab => format!(
                "https://{}/{}/{}/merge_requests/{}",
                self.host, self.owner, self.name, num
            )
            .into(),
            Provider::Bitbucket => format!(
                "https://{}/{}/{}/pull-requests/{}",
                self.host, self.owner, self.name, num
            )
            .into(),
            Provider::Other => EcoString::new(),
        }
    }
    pub fn compare_url(&self, base: &str, head: &str) -> EcoString {
        match self.provider {
            Provider::GitHub | Provider::GitLab => format!(
                "https://{}/{}/{}/compare/{}...{}",
                self.host, self.owner, self.name, base, head
            )
            .into(),
            Provider::Bitbucket => format!(
                "https://{}/{}/{}/branches/compare/{}..{}",
                self.host, self.owner, self.name, head, base
            )
            .into(),
            Provider::Other => EcoString::new(),
        }
    }
}

pub fn format_reference(repo: Option<&Repository>, kind: ReferenceKind, raw: &str) -> EcoString {
    let Some(r) = repo else {
        return raw.into();
    };
    let (segment, display) = match kind {
        ReferenceKind::PullRequest => match r.provider {
            Provider::GitHub => ("pull", raw.trim_start_matches('#')),
            Provider::GitLab => ("merge_requests", raw.trim_start_matches('#')),
            Provider::Bitbucket => ("pull-requests", raw.trim_start_matches('#')),
            Provider::Other => return raw.into(),
        },
        ReferenceKind::Issue => ("issues", raw.trim_start_matches('#')),
        ReferenceKind::Hash => match r.provider {
            Provider::GitHub | Provider::GitLab => ("commit", raw),
            Provider::Bitbucket => ("commits", raw),
            Provider::Other => return raw.into(),
        },
    };
    format!(
        "[{}](https://{}/{}/{}/{}/{})",
        raw, r.host, r.owner, r.name, segment, display
    )
    .into()
}

pub fn format_compare_changes(
    v: Option<&str>,
    from: &str,
    to: &str,
    repo: Option<&Repository>,
) -> Option<EcoString> {
    let r = repo?;
    let head = v.unwrap_or(to);
    let url = match r.provider {
        Provider::GitHub | Provider::GitLab => format!(
            "https://{}/{}/{}/compare/{}...{}",
            r.host, r.owner, r.name, from, head
        ),
        Provider::Bitbucket => format!(
            "https://{}/{}/{}/branches/compare/{}..{}",
            r.host, r.owner, r.name, head, from
        ),
        Provider::Other => return None,
    };
    Some(format!("[compare changes]({})", url).into())
}

impl fmt::Display for Repository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}/{}", self.host, self.owner, self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ssh() {
        let r = Repository::parse("git@github.com:owner/repo.git").unwrap();
        assert_eq!(r.provider, Provider::GitHub);
        assert_eq!(r.owner, "owner");
        assert_eq!(r.name, "repo");
    }

    #[test]
    fn parse_https_git() {
        let r = Repository::parse("https://github.com/owner/repo.git").unwrap();
        assert_eq!(r.provider, Provider::GitHub);
    }

    #[test]
    fn urls_github() {
        let r = Repository::parse("git@github.com:owner/repo.git").unwrap();
        assert!(r.commit_url("abcdef").contains("commit/abcdef"));
        assert!(
            r.compare_url("v1.0.0", "v1.1.0")
                .ends_with("v1.0.0...v1.1.0")
        );
    }

    #[test]
    fn compare_bitbucket() {
        let r = Repository {
            host: "bitbucket.org".into(),
            owner: "o".into(),
            name: "r".into(),
            provider: Provider::Bitbucket,
            original: "".into(),
        };
        assert_eq!(
            r.compare_url("a", "b"),
            "https://bitbucket.org/o/r/branches/compare/b..a"
        );
    }
}
