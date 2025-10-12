use divan::{AllocProfiler, Bencher};
use ecow::EcoString;
use mimalloc_safe::MiMalloc;
use novalyn_core::authors::{AuthorOptions, Authors};
use novalyn_core::changelog::write_or_update_changelog;
use novalyn_core::config::{LoadOptions, load_config};
use novalyn_core::conventional::parse_commit_fast;
use novalyn_core::git::RawCommit;
use novalyn_core::parse::{ParsedCommit, parse_and_classify};
use std::collections::HashMap;
use tempfile::TempDir;

#[global_allocator]
static GLOBAL: AllocProfiler<MiMalloc> = AllocProfiler::new(MiMalloc);

fn generate_synthetic_commits(count: usize) -> Vec<RawCommit> {
    let commit_types = ["feat", "fix", "docs", "style", "refactor", "test", "chore"];
    let scopes = ["api", "ui", "core", "auth", "db"];

    (0..count)
        .map(|i| {
            let commit_type = commit_types[i % commit_types.len()];
            let scope = if i % 3 == 0 {
                format!("({})", scopes[i % scopes.len()])
            } else {
                String::new()
            };
            let breaking = if i % 20 == 0 { "!" } else { "" };
            RawCommit {
                id: format!("commit{:06}", i).into(),
                short_id: format!("c{:06x}", i).into(),
                summary: format!("{}{}{}: implement feature {}", commit_type, scope, breaking, i)
                    .into(),
                body: if i % 10 == 0 {
                    format!(
                        "This is a detailed commit body for commit {}.\n\nIt contains multiple paragraphs.",
                        i
                    )
                    .into()
                } else {
                    String::new().into()
                },
                author_name: format!("Author {}", i % 10).into(),
                author_email: format!("author{}@example.com", i % 10).into(),
                timestamp: 1704110400 + (i as i64 * 3600),
            }
        })
        .collect()
}

/// Benchmark config loading from filesystem
#[divan::bench(args = [1, 5, 10])]
fn config_loading(bencher: Bencher, iterations: usize) {
    let td = TempDir::new().unwrap();

    bencher.bench(|| {
        for _ in 0..iterations {
            let _ = load_config(LoadOptions {
                cwd: td.path(),
                cli_overrides: None,
            });
        }
    });
}

/// Benchmark author collection and deduplication
#[divan::bench(args = [10, 50, 100, 500])]
fn authors_collection(bencher: Bencher, size: usize) {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    let commits = generate_synthetic_commits(size);
    let parsed = parse_and_classify(commits.into(), &cfg);

    bencher.bench(|| {
        let opts = AuthorOptions {
            exclude: vec![].into(),
            hide_author_email: false,
            no_authors: false,
            aliases: HashMap::with_hasher(foldhash::quality::RandomState::default()),
            github_token: None,
            enable_github_aliasing: false,
        };
        Authors::collect(&parsed, &opts)
    });
}

/// Benchmark conventional commit parsing (raw to parsed)
#[divan::bench(args = [10, 50, 100, 500])]
fn conventional_parse(bencher: Bencher, size: usize) {
    let commits = generate_synthetic_commits(size);

    bencher
        .with_inputs(|| commits.clone())
        .bench_values(|commits| {
            for commit in &commits {
                let _ = parse_commit_fast(commit);
            }
        });
}

/// Benchmark changelog file write operations
#[divan::bench(args = [1, 5, 10])]
fn changelog_write(bencher: Bencher, block_count: usize) {
    let td = TempDir::new().unwrap();

    bencher.bench(|| {
        for i in 0..block_count {
            let block: EcoString =
                format!("## v1.{}.0\n\n### Features\n\n* Added feature {}\n", i, i).into();
            let _ = write_or_update_changelog(td.path(), &block);
        }
    });
}

/// Benchmark scope mapping on parsed commits
#[divan::bench(args = [10, 50, 100, 500])]
fn scope_mapping(bencher: Bencher, size: usize) {
    let td = TempDir::new().unwrap();
    let mut cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    // Add scope mappings
    cfg.scope_map.insert("api".into(), "API".into());
    cfg.scope_map.insert("ui".into(), "UI".into());
    cfg.scope_map.insert("core".into(), "Core".into());

    let commits = generate_synthetic_commits(size);

    bencher
        .with_inputs(|| commits.clone())
        .bench_values(|commits| parse_and_classify(commits.into(), &cfg));
}

/// Benchmark issue number extraction from commit messages
#[divan::bench(args = [10, 50, 100])]
fn issue_extraction(bencher: Bencher, size: usize) {
    let commits: Vec<RawCommit> = (0..size)
        .map(|i| RawCommit {
            id: format!("commit{:06}", i).into(),
            short_id: format!("c{:06x}", i).into(),
            summary: format!("feat: add feature #{} #{} #{}", i, i * 2, i * 3).into(),
            body: format!("Closes #{}\nFixes #{}", i * 4, i * 5).into(),
            author_name: "Author".into(),
            author_email: "author@example.com".into(),
            timestamp: 1704110400,
        })
        .collect();

    bencher
        .with_inputs(|| commits.clone())
        .bench_values(|commits| {
            for commit in &commits {
                let _ = parse_commit_fast(commit);
            }
        });
}

fn main() {
    divan::main();
}
