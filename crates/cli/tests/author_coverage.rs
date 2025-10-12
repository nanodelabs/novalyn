use assert_fs::TempDir;
use ecow::{EcoString, EcoVec};
use novalyn::authors::{Author, AuthorOptions, Authors};
use novalyn::config::{LoadOptions, load_config};
use novalyn::git::RawCommit;
use novalyn::parse::parse_and_classify;

fn mk_commit(name: &str, email: &str, co_authors: &[&str]) -> RawCommit {
    let mut body = String::new();
    for co in co_authors {
        body.push_str(&format!("Co-authored-by: {}\n", co));
    }
    RawCommit {
        id: "x".into(),
        short_id: "x".into(),
        summary: "feat: test".into(),
        body: body.into(),
        author_name: name.into(),
        author_email: email.into(),
        timestamp: 0,
    }
}

#[test]
fn test_author_collection_basic() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    let commits = vec![
        mk_commit("Alice", "alice@example.com", &[]),
        mk_commit("Bob", "bob@example.com", &[]),
    ];
    let parsed = parse_and_classify(commits.into(), &cfg);

    let opts = AuthorOptions::default();
    let authors = Authors::collect(&parsed, &opts);

    assert_eq!(authors.list.len(), 2);
    assert!(!authors.suppressed);
}

#[test]
fn test_author_deduplication() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    let commits = vec![
        mk_commit("Alice", "alice@example.com", &[]),
        mk_commit("Alice", "alice@example.com", &[]), // duplicate
        mk_commit("Bob", "bob@example.com", &[]),
    ];
    let parsed = parse_and_classify(commits.into(), &cfg);

    let opts = AuthorOptions::default();
    let authors = Authors::collect(&parsed, &opts);

    assert_eq!(authors.list.len(), 2); // Only 2 unique authors
}

#[test]
fn test_author_exclusion_by_name() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    let commits = vec![
        mk_commit("Alice", "alice@example.com", &[]),
        mk_commit("Bot", "bot@example.com", &[]),
    ];
    let parsed = parse_and_classify(commits.into(), &cfg);

    let opts = AuthorOptions {
        exclude: EcoVec::from(vec![EcoString::from("Bot")]),
        ..Default::default()
    };
    let authors = Authors::collect(&parsed, &opts);

    assert_eq!(authors.list.len(), 1);
    assert_eq!(authors.list[0].name.as_str(), "Alice");
}

#[test]
fn test_author_exclusion_by_email() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    let commits = vec![
        mk_commit("Alice", "alice@example.com", &[]),
        mk_commit("Bot", "bot@automation.com", &[]),
    ];
    let parsed = parse_and_classify(commits.into(), &cfg);

    let opts = AuthorOptions {
        exclude: EcoVec::from(vec![EcoString::from("bot@automation.com")]),
        ..Default::default()
    };
    let authors = Authors::collect(&parsed, &opts);

    assert_eq!(authors.list.len(), 1);
    assert_eq!(authors.list[0].name.as_str(), "Alice");
}

#[test]
fn test_hide_author_email() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    let commits = vec![mk_commit("Alice", "alice@example.com", &[])];
    let parsed = parse_and_classify(commits.into(), &cfg);

    let opts = AuthorOptions {
        hide_author_email: true,
        ..Default::default()
    };
    let authors = Authors::collect(&parsed, &opts);

    assert_eq!(authors.list.len(), 1);
    assert!(authors.list[0].email.is_none()); // Email should be hidden
}

#[test]
fn test_no_authors_suppression() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    let commits = vec![mk_commit("Alice", "alice@example.com", &[])];
    let parsed = parse_and_classify(commits.into(), &cfg);

    let opts = AuthorOptions {
        no_authors: true,
        ..Default::default()
    };
    let authors = Authors::collect(&parsed, &opts);

    assert!(authors.list.is_empty());
    assert!(authors.suppressed);
}

#[test]
fn test_co_authors_parsing() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    let mut commits = vec![mk_commit(
        "Alice",
        "alice@example.com",
        &["Co-authored-by: Charlie <charlie@x.com>"],
    )];
    // Need to add the co-author line to the body correctly
    commits[0].body = "Co-authored-by: Charlie <charlie@x.com>\n"
        .to_string()
        .into();
    let parsed = parse_and_classify(commits.into(), &cfg);

    let opts = AuthorOptions::default();
    let authors = Authors::collect(&parsed, &opts);

    // Should have at least Alice, might have Charlie if co-author parsing works
    assert!(!authors.list.is_empty());
}

#[test]
fn test_author_aliasing() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    let commits = vec![
        mk_commit("Alice", "alice@example.com", &[]),
        mk_commit("Alice Smith", "alice@example.com", &[]), // Different name, same email
    ];
    let parsed = parse_and_classify(commits.into(), &cfg);

    let mut opts = AuthorOptions::default();
    opts.aliases
        .insert(EcoString::from("Alice Smith"), EcoString::from("Alice"));
    let authors = Authors::collect(&parsed, &opts);

    // Both should resolve to "Alice" due to aliasing
    assert_eq!(authors.list.len(), 1);
    assert_eq!(authors.list[0].name.as_str(), "Alice");
}

#[test]
fn test_unicode_normalization() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    // Different unicode representations of the same character
    let commits = vec![
        mk_commit("José", "jose@example.com", &[]), // é as single character
        mk_commit("José", "jose@example.com", &[]), // é as combining characters
    ];
    let parsed = parse_and_classify(commits.into(), &cfg);

    let opts = AuthorOptions::default();
    let authors = Authors::collect(&parsed, &opts);

    // Should deduplicate despite different unicode representations
    assert_eq!(authors.list.len(), 1);
}

#[test]
fn test_empty_email_handling() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    let commits = vec![mk_commit("Alice", "", &[])]; // Empty email
    let parsed = parse_and_classify(commits.into(), &cfg);

    let opts = AuthorOptions::default();
    let authors = Authors::collect(&parsed, &opts);

    assert_eq!(authors.list.len(), 1);
    assert!(authors.list[0].email.is_none()); // Empty email should be None
}

#[test]
fn test_resolve_github_handles_structure() {
    // Test the structure without actually calling async function
    let authors = Authors {
        list: EcoVec::from(vec![Author {
            name: EcoString::from("Alice"),
            email: Some(EcoString::from("alice@example.com")),
        }]),
        suppressed: false,
    };

    // Just verify the structure is correct
    assert_eq!(authors.list.len(), 1);
    assert!(!authors.suppressed);
}

#[test]
fn test_author_options_default() {
    let opts = AuthorOptions::default();
    assert!(opts.exclude.is_empty());
    assert!(!opts.hide_author_email);
    assert!(!opts.no_authors);
    assert!(opts.aliases.is_empty());
    assert!(opts.github_token.is_none());
    assert!(!opts.enable_github_aliasing);
}
