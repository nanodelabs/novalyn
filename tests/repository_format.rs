use changelogen::repository::{
    Provider, ReferenceKind, Repository, format_compare_changes, format_reference,
};

#[test]
fn format_issue_github() {
    let repo = Repository::parse("git@github.com:unjs/changelogen.git").unwrap();
    let s = format_reference(Some(&repo), ReferenceKind::Issue, "#42");
    assert_eq!(s, "[#42](https://github.com/unjs/changelogen/issues/42)");
}

#[test]
fn format_pr_github() {
    let repo = Repository::parse("https://github.com/unjs/changelogen.git").unwrap();
    let s = format_reference(Some(&repo), ReferenceKind::PullRequest, "#7");
    assert_eq!(s, "[#7](https://github.com/unjs/changelogen/pull/7)");
}

#[test]
fn format_hash_bitbucket() {
    let repo = Repository {
        host: "bitbucket.org".into(),
        owner: "o".into(),
        name: "r".into(),
        provider: Provider::Bitbucket,
        original: String::new().into(),
    };
    let s = format_reference(Some(&repo), ReferenceKind::Hash, "abcdef1");
    assert_eq!(s, "[abcdef1](https://bitbucket.org/o/r/commits/abcdef1)");
}

#[test]
fn reference_without_repo() {
    let s = format_reference(None, ReferenceKind::Issue, "#99");
    assert_eq!(s, "#99");
}

#[test]
fn compare_links() {
    let gh = Repository::parse("git@github.com:unjs/changelogen.git").unwrap();
    let cmp = format_compare_changes(Some("v1.2.0"), "v1.1.0", "deadbeef", Some(&gh)).unwrap();
    assert!(cmp.contains(
        "[compare changes](https://github.com/unjs/changelogen/compare/v1.1.0...v1.2.0)"
    ));

    let bb = Repository {
        host: "bitbucket.org".into(),
        owner: "o".into(),
        name: "r".into(),
        provider: Provider::Bitbucket,
        original: String::new().into(),
    };
    let cmp_bb = format_compare_changes(Some("v1.2.0"), "v1.1.0", "deadbeef", Some(&bb)).unwrap();
    assert_eq!(
        cmp_bb,
        "[compare changes](https://bitbucket.org/o/r/branches/compare/v1.2.0..v1.1.0)"
    );
}
