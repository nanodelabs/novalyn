use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    GitHub,
    GitLab,
    Bitbucket,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    pub host: String,
    pub owner: String,
    pub name: String,
    pub provider: Provider,
    pub original: String,
}

impl Repository {
    pub fn parse(remote: &str) -> Option<Self> {
        // Try SSH: git@host:owner/name(.git)
        if let Some(rest) = remote.strip_prefix("git@") {
            let mut parts = rest.splitn(2, ':');
            let host = parts.next()?.to_string();
            let path = parts.next()?;
            return Self::from_host_path(host, path, remote);
        }
        // SSH alternative: ssh://git@host/owner/name(.git)
        if let Some(stripped) = remote.strip_prefix("ssh://git@")
            && let Some((host, path)) = stripped.split_once('/')
        {
            return Self::from_host_path(host.to_string(), path, remote);
        }
        // HTTPS: https://host/owner/name(.git)
        if let Some(stripped) = remote.strip_prefix("https://")
            && let Some((host, path)) = stripped.split_once('/')
        {
            return Self::from_host_path(host.to_string(), path, remote);
        }
        // HTTP (rare)
        if let Some(stripped) = remote.strip_prefix("http://")
            && let Some((host, path)) = stripped.split_once('/')
        {
            return Self::from_host_path(host.to_string(), path, remote);
        }
        None
    }

    fn from_host_path(host: String, path: &str, original: &str) -> Option<Self> {
        let path = path.trim_end_matches('/').trim_end_matches(".git");
        let mut segs = path.split('/');
        let owner = segs.next()?.to_string();
        let name = segs.next()?.to_string();
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
            original: original.to_string(),
        })
    }

    pub fn commit_url(&self, sha: &str) -> String {
        match self.provider {
            Provider::GitHub | Provider::GitLab => format!(
                "https://{}/{}/{}/commit/{}",
                self.host, self.owner, self.name, sha
            ),
            Provider::Bitbucket => format!(
                "https://{}/{}/{}/commits/{}",
                self.host, self.owner, self.name, sha
            ),
            Provider::Other => String::new(),
        }
    }
    pub fn tag_url(&self, tag: &str) -> String {
        match self.provider {
            Provider::GitHub | Provider::GitLab => format!(
                "https://{}/{}/{}/releases/tag/{}",
                self.host, self.owner, self.name, tag
            ),
            Provider::Bitbucket => format!(
                "https://{}/{}/{}/src/{}",
                self.host, self.owner, self.name, tag
            ),
            Provider::Other => String::new(),
        }
    }
    pub fn issue_url(&self, num: u64) -> String {
        match self.provider {
            Provider::GitHub | Provider::GitLab => format!(
                "https://{}/{}/{}/issues/{}",
                self.host, self.owner, self.name, num
            ),
            Provider::Bitbucket => format!(
                "https://{}/{}/{}/issues/{}",
                self.host, self.owner, self.name, num
            ),
            Provider::Other => String::new(),
        }
    }
    pub fn pr_url(&self, num: u64) -> String {
        match self.provider {
            Provider::GitHub => format!(
                "https://{}/{}/{}/pull/{}",
                self.host, self.owner, self.name, num
            ),
            Provider::GitLab => format!(
                "https://{}/{}/{}/merge_requests/{}",
                self.host, self.owner, self.name, num
            ),
            Provider::Bitbucket => format!(
                "https://{}/{}/{}/pull-requests/{}",
                self.host, self.owner, self.name, num
            ),
            Provider::Other => String::new(),
        }
    }
    pub fn compare_url(&self, base: &str, head: &str) -> String {
        match self.provider {
            Provider::GitHub | Provider::GitLab => format!(
                "https://{}/{}/{}/compare/{}...{}",
                self.host, self.owner, self.name, base, head
            ),
            Provider::Bitbucket => format!(
                "https://{}/{}/{}/branches/compare/{}..{}",
                self.host, self.owner, self.name, head, base
            ),
            Provider::Other => String::new(),
        }
    }
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
