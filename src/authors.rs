use crate::parse::ParsedCommit;
use ecow::{EcoString, EcoVec};
use once_cell::sync::Lazy;
use unicode_normalization::UnicodeNormalization;

type FastHashMap<K, V> = std::collections::HashMap<K, V, foldhash::quality::RandomState>;
type FastHashSet<T> = std::collections::HashSet<T, foldhash::quality::RandomState>;

// Reusable hash builder to avoid allocation overhead
static HASH_BUILDER: Lazy<foldhash::quality::RandomState> =
    Lazy::new(foldhash::quality::RandomState::default);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Author {
    pub name: EcoString,
    pub email: Option<EcoString>,
}

#[derive(Debug, Clone, Default)]
pub struct Authors {
    pub list: EcoVec<Author>,
    pub suppressed: bool,
}

#[derive(Debug, Clone)]
pub struct AuthorOptions {
    pub exclude: EcoVec<EcoString>, // names or emails exact match
    pub hide_author_email: bool,    // redact email if true
    pub no_authors: bool,           // suppress entirely
    pub aliases: FastHashMap<EcoString, EcoString>, // map old identity to new (name or email)
    pub github_token: Option<String>, // GitHub token for email->handle resolution
    pub enable_github_aliasing: bool, // whether to resolve emails to @handles
}

impl Default for AuthorOptions {
    fn default() -> Self {
        Self {
            exclude: EcoVec::new(),
            hide_author_email: false,
            no_authors: false,
            aliases: FastHashMap::with_hasher(HASH_BUILDER.clone()),
            github_token: None,
            enable_github_aliasing: false,
        }
    }
}

impl Authors {
    pub fn collect(commits: &[ParsedCommit], opts: &AuthorOptions) -> Self {
        if opts.no_authors {
            return Authors {
                list: EcoVec::new(),
                suppressed: true,
            };
        }
        let mut seen = FastHashSet::with_hasher(HASH_BUILDER.clone());
        let mut out = EcoVec::with_capacity(commits.len());
        for c in commits {
            // primary author
            push_author(
                &mut out,
                &mut seen,
                &c.raw.author_name,
                &c.raw.author_email,
                opts,
            );
            // co-authors lines like "Name <email>" already captured in ParsedCommit.co_authors
            for line in &c.co_authors {
                if let Some((name, email)) = parse_co_author_line(line) {
                    push_author(&mut out, &mut seen, name, email, opts);
                }
            }
        }
        Authors {
            list: out,
            suppressed: false,
        }
    }

    /// Resolve email addresses to GitHub handles using GitHub API.
    /// This modifies author names in place, replacing emails with @handles when found.
    pub async fn resolve_github_handles(&mut self, token: &str) -> Result<(), String> {
        use crate::github::get_username_from_email;

        // Make EcoVec mutable to iterate and modify
        let authors_vec = self.list.make_mut();
        for author in authors_vec.iter_mut() {
            if let Some(ref email) = author.email {
                if let Ok(Some(handle)) =
                    get_username_from_email(email.as_str(), Some(token), None).await
                {
                    // Replace name with GitHub handle
                    author.name = EcoString::from(handle);
                }
            }
        }
        Ok(())
    }
}

fn normalize(s: &str) -> EcoString {
    EcoString::from(s.nfc().collect::<String>())
}

fn excluded(opts: &AuthorOptions, name: &EcoString, email: Option<&EcoString>) -> bool {
    if opts.exclude.iter().any(|e| e == name) {
        return true;
    }
    if let Some(e) = email {
        if opts.exclude.iter().any(|x| x == e) {
            return true;
        }
    }
    false
}

fn push_author<'a>(
    out: &mut EcoVec<Author>,
    seen: &mut FastHashSet<(EcoString, Option<EcoString>)>,
    name: &'a str,
    email: &'a str,
    opts: &AuthorOptions,
) {
    let mut name_n = normalize(name.trim());
    let mut email_n = if email.trim().is_empty() {
        None
    } else {
        Some(normalize(email.trim()))
    };

    // Apply aliases
    if let Some(alias) = opts.aliases.get(&name_n) {
        name_n = alias.clone();
    }
    if let Some(ref e) = email_n {
        if let Some(alias) = opts.aliases.get(e) {
            email_n = Some(alias.clone());
        }
    }

    if excluded(opts, &name_n, email_n.as_ref()) {
        return;
    }
    let key = (name_n.clone(), email_n.clone());
    if !seen.insert(key) {
        return;
    }
    let email_final = if opts.hide_author_email {
        None
    } else {
        email_n
    };
    out.push(Author {
        name: name_n,
        email: email_final,
    });
}

fn parse_co_author_line(line: &str) -> Option<(&str, &str)> {
    // Format: Name <email>
    let line = line.trim();
    if let Some(start) = line.rfind('<')
        && let Some(end) = line.rfind('>')
        && end > start
    {
        let name = line[..start].trim();
        let email = &line[start + 1..end];
        return Some((name, email));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::RawCommit;

    fn mk_commit(name: &str, email: &str, co: &[&str]) -> ParsedCommit {
        ParsedCommit {
            raw: RawCommit {
                id: "1".into(),
                short_id: "1".into(),
                summary: "feat: something".into(),
                body: String::new(),
                author_name: name.into(),
                author_email: email.into(),
                timestamp: 0,
            },
            r#type: "feat".into(),
            scope: None,
            description: "something".into(),
            body: String::new(),
            footers: vec![],
            breaking: false,
            issues: vec![],
            co_authors: co.iter().map(|s| s.to_string()).collect(),
            type_cfg: None,
            index: 0,
        }
    }

    #[test]
    fn collects_and_dedups() {
        let commits = vec![
            mk_commit("Alice", "alice@example.com", &[]),
            mk_commit("Bob", "bob@example.com", &[]),
            mk_commit("Alice", "alice@example.com", &[]),
        ];
        let a = Authors::collect(&commits, &AuthorOptions::default());
        assert_eq!(a.list.len(), 2);
    }

    #[test]
    fn co_authors_parsed() {
        let commits = vec![mk_commit(
            "Alice",
            "alice@example.com",
            &["Charlie <charlie@x.com>"],
        )];
        let a = Authors::collect(&commits, &AuthorOptions::default());
        assert_eq!(a.list.len(), 2);
    }

    #[test]
    fn exclusion_and_hide_email() {
        let commits = vec![mk_commit("Ålice", "alice@example.com", &[])];
        let a = Authors::collect(
            &commits,
            &AuthorOptions {
                exclude: EcoVec::from(vec![EcoString::from("Ålice")]),
                ..Default::default()
            },
        );
        assert!(a.list.is_empty());
        let commits = vec![mk_commit("Dana", "dana@example.com", &[])];
        let a2 = Authors::collect(
            &commits,
            &AuthorOptions {
                hide_author_email: true,
                ..Default::default()
            },
        );
        assert_eq!(a2.list[0].email, None);
    }

    #[test]
    fn author_aliasing() {
        let mut aliases = FastHashMap::with_hasher(foldhash::quality::RandomState::default());
        aliases.insert(
            EcoString::from("old@example.com"),
            EcoString::from("new@example.com"),
        );
        aliases.insert(EcoString::from("OldName"), EcoString::from("NewName"));

        let commits = vec![
            mk_commit("OldName", "old@example.com", &[]),
            mk_commit("NewName", "new@example.com", &[]),
        ];

        let a = Authors::collect(
            &commits,
            &AuthorOptions {
                aliases,
                ..Default::default()
            },
        );

        // Should be deduplicated to one author after aliasing
        assert_eq!(a.list.len(), 1);
        assert_eq!(a.list[0].name, "NewName");
        assert_eq!(a.list[0].email, Some(EcoString::from("new@example.com")));
    }
}
