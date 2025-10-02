use crate::parse::ParsedCommit;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Authors {
    pub list: Vec<Author>,
    pub suppressed: bool,
}

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct AuthorOptions {
    pub exclude: Vec<String>,             // names or emails exact match
    pub hide_author_email: bool,          // redact email if true
    pub no_authors: bool,                 // suppress entirely
    pub aliases: HashMap<String, String>, // map old identity to new (name or email)
}

impl Authors {
    pub fn collect(commits: &[ParsedCommit], opts: &AuthorOptions) -> Self {
        if opts.no_authors {
            return Authors {
                list: Vec::new(),
                suppressed: true,
            };
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut out: Vec<Author> = Vec::new();
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
}

fn normalize(s: &str) -> String {
    s.nfc().collect::<String>()
}

fn excluded(opts: &AuthorOptions, name: &str, email: Option<&str>) -> bool {
    let target_name = normalize(name);
    if opts.exclude.iter().any(|e| e == &target_name) {
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
    out: &mut Vec<Author>,
    seen: &mut std::collections::BTreeSet<(String, Option<String>)>,
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

    if excluded(opts, &name_n, email_n.as_deref()) {
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
                exclude: vec!["Ålice".into()],
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
        let mut aliases = HashMap::new();
        aliases.insert("old@example.com".to_string(), "new@example.com".to_string());
        aliases.insert("OldName".to_string(), "NewName".to_string());

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
        assert_eq!(a.list[0].email, Some("new@example.com".to_string()));
    }
}
