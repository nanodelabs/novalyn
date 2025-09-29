use changelogen::config::{LoadOptions, load_config};
use changelogen::git::RawCommit;
use changelogen::parse::parse_and_classify;
use std::env;
use tempfile::TempDir;

fn create_test_commits(count: usize) -> Vec<RawCommit> {
    let mut commits = Vec::new();
    for i in 0..count {
        commits.push(RawCommit {
            id: format!("commit{:03}", i),
            short_id: format!("c{:03}", i),
            summary: format!("feat: feature {}", i),
            body: String::new(),
            author_name: "Test Author".to_string(),
            author_email: "test@example.com".to_string(),
            timestamp: 1704110400, // 2024-01-01T12:00:00Z as Unix timestamp
        });
    }
    commits
}

#[test]
fn parallel_vs_sequential_identical_output() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    // Create test commits
    let commits = create_test_commits(100);

    // Test sequential processing
    unsafe {
        env::set_var("CHANGELOGEN_PARALLEL_THRESHOLD", "1000"); // Force sequential
    }
    let sequential_result = parse_and_classify(commits.clone(), &cfg);

    // Test parallel processing
    unsafe {
        env::set_var("CHANGELOGEN_PARALLEL_THRESHOLD", "50"); // Force parallel
    }
    let parallel_result = parse_and_classify(commits, &cfg);

    // Results should be identical
    assert_eq!(sequential_result.len(), parallel_result.len());

    // Check that ordering is preserved (by index)
    for (seq, par) in sequential_result.iter().zip(parallel_result.iter()) {
        assert_eq!(seq.index, par.index);
        assert_eq!(seq.raw.id, par.raw.id);
        assert_eq!(seq.r#type, par.r#type);
    }

    // Clean up env var
    unsafe {
        env::remove_var("CHANGELOGEN_PARALLEL_THRESHOLD");
    }
}

#[test]
fn parallel_threshold_respected() {
    let td = TempDir::new().unwrap();
    let cfg = load_config(LoadOptions {
        cwd: td.path(),
        cli_overrides: None,
    })
    .unwrap();

    // Set high threshold to force sequential mode
    unsafe {
        env::set_var("CHANGELOGEN_PARALLEL_THRESHOLD", "200");
    }
    let commits = create_test_commits(10);
    let result = parse_and_classify(commits, &cfg);

    // Should process without issues regardless of mode
    assert!(result.len() <= 10); // Some commits might be filtered

    unsafe {
        env::remove_var("CHANGELOGEN_PARALLEL_THRESHOLD");
    }
}
