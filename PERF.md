# Performance Documentation

This document tracks the performance characteristics of changelogen-rs and provides benchmark results for key operations.

## Benchmark Suite

The project uses [divan](https://github.com/nvzqz/divan) for microbenchmarking. Benchmarks are located in `benches/parse_performance.rs`.

### Available Benchmarks

1. **parse_sequential**: Sequential commit parsing (baseline)
   - Measures commit parsing and classification in single-threaded mode
   - Tested with 10, 50, 100, and 500 commits

2. **parse_parallel**: Parallel commit parsing
   - Measures commit parsing using rayon parallel processing
   - Tested with 50, 100, and 500 commits
   - Uses a threshold of 10 commits to enable parallelism

3. **version_inference**: Semantic version inference
   - Measures the time to infer version bumps from parsed commits
   - Includes major/minor/patch detection and pre-1.0 adjustments
   - Tested with 10, 50, 100, and 500 commits

4. **render_block**: Markdown changelog rendering
   - Measures markdown generation from parsed commits
   - Includes section grouping, formatting, and reference linking
   - Tested with 10, 50, 100, and 500 commits

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench parse_sequential

# Run with specific size
cargo bench -- 100

# Save results for comparison
cargo bench -- --save-baseline main
```

## Baseline Results

> **Note**: Baseline results will be updated after initial release. Run `cargo bench` locally to see performance on your hardware.

Benchmark results depend heavily on:
- CPU architecture and core count
- Available memory
- Git repository size and history depth
- Commit message complexity

### Expected Characteristics

Based on the implementation:

1. **Parsing Performance**
   - Sequential parsing: O(n) where n = commit count
   - Parallel parsing: O(n/cores) for n > 50 commits (configurable threshold)
   - Regex-based conventional commit parsing with git-conventional fallback

2. **Version Inference**
   - O(n) scan through commits to find highest semver impact
   - Early termination on major breaking change detection
   - Constant-time version bump calculation

3. **Rendering**
   - O(n) for commit grouping by type
   - O(n log n) for deterministic sorting within groups
   - Linear string concatenation with pre-allocated buffers

### Parallel Processing Threshold

The `CHANGELOGEN_PARALLEL_THRESHOLD` environment variable controls when to use parallel processing:

```bash
# Default: 50 commits
CHANGELOGEN_PARALLEL_THRESHOLD=50 changelogen release

# Always sequential (useful for debugging)
CHANGELOGEN_PARALLEL_THRESHOLD=10000 changelogen release

# Aggressive parallelism
CHANGELOGEN_PARALLEL_THRESHOLD=10 changelogen release
```

**Recommendation**: The default threshold of 50 commits provides good balance between:
- Overhead of thread spawning and synchronization
- Benefits of parallel processing on multi-core systems

For repositories with consistent commit rates, the default is optimal. Adjust only if profiling shows benefit.

## Performance Goals

### Parity with JavaScript Version

Target: **No more than 10% performance regression** compared to @unjs/changelogen on equivalent operations.

Key areas:
- Commit parsing should be faster due to compiled Rust code
- Git operations use libgit2 (C library) vs nodegit, similar performance expected
- Markdown rendering should be comparable

### Optimization Opportunities

Potential areas for future optimization (evaluated via benchmarks):

1. **String allocation**: Pre-allocate string buffers based on commit count
2. **Parallel rendering**: Currently sequential, could parallelize section rendering
3. **Caching**: Memoize regex compilation and provider detection
4. **Memory pools**: Reuse allocations across multiple operations

### dashmap Evaluation

Task 100 requires evaluating dashmap for concurrent hash maps. Current implementation uses standard HashMap.

**Evaluation criteria**: Keep dashmap only if it provides >5% improvement in parallel parsing benchmarks.

**Status**: Pending benchmark comparison. If benefit is <5%, remove dashmap dependency to minimize footprint.

## Memory Usage

No formal memory profiling yet. Expected characteristics:

- **Commit storage**: ~200-500 bytes per commit (RawCommit struct)
- **Parsed commits**: ~300-600 bytes per commit (ParsedCommit with metadata)
- **Peak usage**: ~2x commit storage during parsing (raw + parsed)
- **Changelog**: Linear with output size, pre-allocated buffers minimize fragmentation

For large repositories (10,000+ commits):
- Expected peak: ~10-20 MB for data structures
- Git operations via libgit2: Additional ~50-100 MB depending on repo

## Profiling

For detailed profiling:

```bash
# CPU profiling with flamegraph
cargo flamegraph --bench parse_performance

# Memory profiling (requires valgrind)
cargo build --release --bench parse_performance
valgrind --tool=massif target/release/deps/parse_performance-*

# Time profiling
cargo build --release
time ./target/release/changelogen release --dry-run
```

## CI Performance Tracking

Benchmark results are not currently tracked in CI but can be added using:
- `cargo-criterion` for regression detection
- GitHub Actions artifacts for result history
- Automated alerts on >10% performance regression

See issue/PR for CI integration: TBD

## Contributing Performance Improvements

When submitting performance optimizations:

1. **Benchmark before and after**: Use `cargo bench -- --save-baseline`
2. **Profile bottlenecks**: Use `cargo flamegraph` or `perf`
3. **Document trade-offs**: Speed vs. memory vs. maintainability
4. **Preserve correctness**: All tests must still pass
5. **Update this doc**: Add new benchmarks or update baselines

## Resources

- [divan benchmark framework](https://github.com/nvzqz/divan)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph)
- [Criterion.rs](https://github.com/bheisler/criterion.rs) (alternative to divan)
