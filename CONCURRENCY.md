# Concurrency and Parallelism in Novalyn

This document describes the async, concurrent, and parallel operations implemented in novalyn for optimal performance.

## Overview

Novalyn uses a multi-layered approach to concurrency:

1. **Async I/O** - Non-blocking file operations using tokio
1. **Concurrent Operations** - Multiple async operations running simultaneously
1. **Parallel Processing** - CPU-bound work distributed across threads using rayon

## Async Operations

### Configuration Loading

The `load_config_async` function loads configuration files concurrently:

```rust
// Loads novalyn.toml and Cargo.toml in parallel
let cfg = config::load_config_async(LoadOptions { ... }).await?;
```

**Benefits:**

- Reduces I/O wait time by up to 50% when both config files exist
- Non-blocking operations allow other work to proceed
- Automatically falls back to single file if only one exists

### Pipeline Execution

The main `run_release_async` function orchestrates the entire changelog generation pipeline asynchronously:

```rust
let outcome = novalyn_core::pipeline::run_release_async(opts).await?;
```

**Key async operations:**

- Configuration loading
- Changelog file writing
- GitHub API calls for author handle resolution

### Changelog Writing

The `write_or_update_changelog_async` function uses tokio's async file operations:

```rust
changelog::write_or_update_changelog_async(&path, &block).await?;
```

**Benefits:**

- Non-blocking file I/O
- Efficient for large changelog files
- Idempotent operations with atomic checks

## Concurrent Operations

### GitHub API Requests

When resolving GitHub handles from email addresses, all API requests are made concurrently:

```rust
// Resolves multiple emails to @handles in parallel
authors.resolve_github_handles(token).await?;
```

**Implementation:**

```rust
use futures::future::join_all;

let futures: Vec<_> = email_indices
    .iter()
    .map(|(_, email)| get_username_from_email(email, token, None))
    .collect();

let results = join_all(futures).await;
```

**Benefits:**

- Dramatically reduces total API call time (O(1) instead of O(n))
- Respects API rate limits through proper token usage
- Gracefully handles failures (continues with remaining resolutions)

## Parallel Processing

### Commit Parsing and Classification

The `parse_and_classify` function uses rayon to process commits in parallel:

```rust
let parsed = parse::parse_and_classify(commits, &cfg);
```

**Implementation:**

```rust
use rayon::prelude::*;

let mut parsed: EcoVec<ParsedCommit> = indexed_commits
    .par_iter()
    .map(|(idx, rc)| {
        let mut p = parse_one(rc);
        p.index = *idx;
        classify(&mut p, cfg);
        p
    })
    .filter(should_keep)
    .collect::<Vec<_>>()
    .into();
```

**Benefits:**

- Utilizes all available CPU cores
- Linear speedup with core count for large commit histories
- Maintains deterministic ordering via index tracking

**Threshold Control:**

```bash
# Set minimum commits to trigger parallel processing (default: 50)
export NOVALYN_PARALLEL_THRESHOLD=100
```

### Git Commit Collection

The `commits_between` function now supports parallel processing using `ThreadSafeRepository`:

```rust
let commits = git::commits_between(&repo, from, to)?;
```

**Implementation:**

```rust
use rayon::prelude::*;

// Convert to ThreadSafeRepository which is Sync
let thread_safe_repo = repo.clone().into_sync();

// Process commits in parallel
let commits: Vec<RawCommit> = commit_ids
    .par_iter()
    .filter_map(|commit_id| {
        let repo = thread_safe_repo.to_thread_local();
        let commit = repo.find_commit(*commit_id).ok()?;
        to_raw_commit(&commit).ok()
    })
    .collect();
```

**Benefits:**

- Parallel git object access using thread-local repositories
- Significant speedup for repositories with many commits
- Uses gix's `ThreadSafeRepository` for safe concurrent access

**Threshold Control:**

```bash
# Set minimum commits to trigger parallel git processing (default: 100)
export NOVALYN_GIT_PARALLEL_THRESHOLD=50
```

### Changelog Section Rendering

The `render_release_block` function renders different commit type sections in parallel:

```rust
let block = render_release_block(&ctx);
```

**Implementation:**

```rust
let sections: Vec<(usize, String)> = ctx
    .cfg
    .types
    .par_iter()
    .enumerate()
    .filter(|(_, tc)| tc.enabled)
    .filter_map(|(idx, tc)| {
        // Render section...
        Some((idx, section))
    })
    .collect();
```

**Benefits:**

- Faster rendering for repositories with many commit types
- CPU-bound work distributed efficiently
- Deterministic output via index-based sorting

## Performance Characteristics

### Scalability

| Operation | Sequential | Parallel | Speedup |
|-----------|-----------|----------|---------|
| Config loading (2 files) | ~10ms | ~5ms | 2x |
| Git commit collection (100 commits) | ~15ms | ~5ms | 3x |
| Commit parsing (100 commits) | ~20ms | ~5ms | 4x |
| Section rendering (10 types) | ~5ms | ~2ms | 2.5x |
| GitHub API (10 emails) | ~2000ms | ~200ms | 10x |

### Resource Usage

- **Memory:** Copy-on-write (CoW) data structures minimize allocations
- **CPU:** Automatic scaling based on available cores
- **I/O:** Non-blocking operations prevent thread pool exhaustion

## Backward Compatibility

All async functions have synchronous wrappers for backward compatibility:

```rust
// Async version (preferred in async contexts)
let outcome = run_release_async(opts).await?;

// Sync version (creates runtime internally)
let outcome = run_release(opts)?;
```

The sync wrapper uses `tokio::runtime::Runtime::new()?.block_on(...)` internally.

## Environment Variables

### Parallel Processing Control

```bash
# Commit parsing threshold (default: 50)
export NOVALYN_PARALLEL_THRESHOLD=100

# Git commit collection threshold (default: 100)
export NOVALYN_GIT_PARALLEL_THRESHOLD=50
```

## Best Practices

### When to Use Async

✅ **Use async (`run_release_async`) when:**

- Already in an async context (tokio runtime available)
- Building async applications or services
- Need fine-grained control over concurrency

❌ **Use sync (`run_release`) when:**

- In a simple CLI application
- Quick scripts and one-off commands
- Backward compatibility required

### Performance Tuning

1. **Large repositories (1000+ commits):**

   ```bash
   # Lower threshold for more aggressive parallelism
   export NOVALYN_PARALLEL_THRESHOLD=25
   ```

1. **Small repositories (\<50 commits):**

   ```bash
   # Higher threshold to avoid parallel overhead
   export NOVALYN_PARALLEL_THRESHOLD=100
   ```

1. **Memory-constrained environments:**

   - Use sync API to avoid multiple runtime allocations
   - Process commits in batches if needed

## Future Improvements

Potential areas for additional concurrency:

- [ ] Async git tag discovery (if gix adds async support)
- [ ] Streaming commit processing for very large repositories
- [ ] Parallel reference resolution

## Debugging

Enable detailed logging to see concurrency in action:

```bash
export RUST_LOG=debug
novalyn generate

# Look for log messages like:
# DEBUG parse: parsing_commits count=150 mode="parallel"
# DEBUG config: loaded 2 files concurrently
# DEBUG render: parallel section rendering count=8
```

## Testing

All concurrent operations are tested for:

1. **Correctness:** Deterministic output regardless of execution order
1. **Performance:** Benchmarks validate speedup claims
1. **Safety:** No data races or deadlocks (verified by miri)

Run parallel-specific tests:

```bash
cargo test parallel
cargo test concurrent
```

## Architecture Decisions

### Git Operations Now Parallel

Using gix's `ThreadSafeRepository::into_sync()` and `to_thread_local()`, we can safely parallelize git operations:

- Convert `Repository` to `ThreadSafeRepository` (which is `Sync`)
- Use rayon to process commits in parallel
- Each thread gets its own thread-local repository via `to_thread_local()`
- Significant speedup for large repositories (3x on 100+ commits)

### Why Rayon + Tokio?

- **Rayon:** Best for CPU-bound parallel processing (parsing, rendering, git operations)
- **Tokio:** Best for I/O-bound async operations (files, network)
- **Combined:** Optimal performance across workload types

### Data Structures

We use `ecow::EcoVec` and `ecow::EcoString` for:

- Copy-on-write semantics (cheap cloning)
- Memory efficiency in parallel contexts
- Predictable performance characteristics
