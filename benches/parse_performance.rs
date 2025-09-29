use changelogen::config::{load_config, LoadOptions};
use changelogen::git::RawCommit;
use changelogen::parse::parse_and_classify;
use divan::Bencher;
use std::env;
use tempfile::TempDir;

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
                id: format!("commit{:06}", i),
                short_id: format!("c{:06x}", i),
                summary: format!("{}{}{}: implement feature {}", commit_type, scope, breaking, i),
                body: if i % 10 == 0 {
                    format!("This is a detailed commit body for commit {}.\n\nIt contains multiple paragraphs and explains the changes in detail.", i)
                } else {
                    String::new()
                },
                author_name: format!("Author {}", i % 10),
                author_email: format!("author{}@example.com", i % 10),
                timestamp: 1704110400 + (i as i64 * 3600), // Hourly commits
            }
        })
        .collect()
}

#[divan::bench(args = [10, 50, 100, 500])]
fn parse_sequential(bencher: Bencher, size: usize) {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    }).unwrap();
    
    let commits = generate_synthetic_commits(size);
    
    bencher
        .with_inputs(|| {
            unsafe { env::set_var("CHANGELOGEN_PARALLEL_THRESHOLD", "10000"); } // Force sequential
            commits.clone()
        })
        .bench_values(|commits| {
            parse_and_classify(commits, &cfg)
        });
}

#[divan::bench(args = [50, 100, 500])]
fn parse_parallel(bencher: Bencher, size: usize) {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    }).unwrap();
    
    let commits = generate_synthetic_commits(size);
    
    bencher
        .with_inputs(|| {
            unsafe { env::set_var("CHANGELOGEN_PARALLEL_THRESHOLD", "10"); } // Force parallel
            commits.clone()
        })
        .bench_values(|commits| {
            parse_and_classify(commits, &cfg)
        });
}

#[divan::bench(args = [10, 50, 100, 500])]
fn version_inference(bencher: Bencher, size: usize) {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    }).unwrap();

    let commits = generate_synthetic_commits(size);
    let parsed = parse_and_classify(commits, &cfg);
    let previous_version = semver::Version::new(1, 0, 0);
    
    bencher
        .with_inputs(|| (previous_version.clone(), parsed.clone()))
        .bench_values(|(prev_version, parsed_commits)| {
            changelogen::parse::infer_version(&prev_version, &parsed_commits, None)
        });
}

#[divan::bench(args = [10, 50, 100, 500])]
fn render_block(bencher: Bencher, size: usize) {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    }).unwrap();

    let current_version = semver::Version::new(1, 0, 0);
    let previous_version = semver::Version::new(0, 9, 0);

    let commits = generate_synthetic_commits(size);
    let parsed = parse_and_classify(commits, &cfg);
    
    bencher
        .with_inputs(|| {
            changelogen::render::RenderContext {
                commits: &parsed,
                version: &current_version,
                previous_version: Some(&previous_version),
                authors: None, // Skip authors for benchmark
                repo: None,    // Skip repo for benchmark
                cfg: &cfg,
                previous_tag: Some("v0.9.0"),
                current_ref: "HEAD",
            }
        })
        .bench_values(|rc| {
            changelogen::render::render_release_block(&rc)
        });
}

fn main() {
    divan::main();
}