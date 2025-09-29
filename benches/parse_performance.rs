use changelogen::config::{load_config, LoadOptions};
use changelogen::git::RawCommit;
use changelogen::parse::parse_and_classify;
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::{env, hint::black_box};
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

fn bench_parse_sequential_vs_parallel(c: &mut Criterion) {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    }).unwrap();

    let mut group = c.benchmark_group("parse_sequential_vs_parallel");
    
    for size in [10, 50, 100, 500].iter() {
        let commits = generate_synthetic_commits(*size);
        
        // Sequential benchmark
        group.bench_with_input(
            BenchmarkId::new("sequential", size), 
            size, 
            |b, &_size| {
                unsafe { env::set_var("CHANGELOGEN_PARALLEL_THRESHOLD", "10000"); } // Force sequential
                b.iter(|| {
                    parse_and_classify(black_box(commits.clone()), black_box(&cfg))
                });
            }
        );
        
        // Parallel benchmark (only for larger sizes)
        if *size >= 50 {
            group.bench_with_input(
                BenchmarkId::new("parallel", size), 
                size, 
                |b, &_size| {
                    unsafe { env::set_var("CHANGELOGEN_PARALLEL_THRESHOLD", "10"); } // Force parallel
                    b.iter(|| {
                        parse_and_classify(black_box(commits.clone()), black_box(&cfg))
                    });
                }
            );
        }
    }
    
    group.finish();
    unsafe { env::remove_var("CHANGELOGEN_PARALLEL_THRESHOLD"); }
}

fn bench_version_inference(c: &mut Criterion) {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    }).unwrap();

    let mut group = c.benchmark_group("version_inference");
    
    for size in [10, 50, 100, 500].iter() {
        let commits = generate_synthetic_commits(*size);
        let parsed = parse_and_classify(commits, &cfg);
        let previous_version = semver::Version::new(1, 0, 0);
        
        group.bench_with_input(
            BenchmarkId::new("version_bump", size), 
            size, 
            |b, &_size| {
                b.iter(|| {
                    changelogen::parse::infer_version(
                        black_box(&previous_version), 
                        black_box(&parsed), 
                        None
                    )
                });
            }
        );
    }
    
    group.finish();
}

fn bench_render_block(c: &mut Criterion) {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    }).unwrap();

    let current_version = semver::Version::new(1, 0, 0);
    let previous_version = semver::Version::new(0, 9, 0);

    let mut group = c.benchmark_group("render_block");
    
    for size in [10, 50, 100, 500].iter() {
        let commits = generate_synthetic_commits(*size);
        let parsed = parse_and_classify(commits, &cfg);
        
        group.bench_with_input(
            BenchmarkId::new("render", size), 
            size, 
            |b, &_size| {
                b.iter(|| {
                    let rc = changelogen::render::RenderContext {
                        commits: black_box(&parsed),
                        version: black_box(&current_version),
                        previous_version: Some(black_box(&previous_version)),
                        authors: None, // Skip authors for benchmark
                        repo: None,    // Skip repo for benchmark
                        cfg: black_box(&cfg),
                        previous_tag: Some("v0.9.0"),
                        current_ref: "HEAD",
                    };
                    changelogen::render::render_release_block(black_box(&rc))
                });
            }
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_parse_sequential_vs_parallel, bench_version_inference, bench_render_block);
criterion_main!(benches);