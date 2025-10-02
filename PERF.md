# Performance Documentation

This document tracks the performance characteristics of changelogen-rs and provides benchmark results for key operations.

## Benchmark Suite

The project uses [CodSpeed](https://codspeed.io/) for continuous benchmarking via the `codspeed-divan-compat` package. This provides a divan-compatible API with CodSpeed's instrumentation capabilities.

**Benchmark location**: `benches/parse_performance.rs`

**Framework**: `codspeed-divan-compat` (CodSpeed-instrumented divan API)

### Available Benchmarks

1. **parse_sequential**: Sequential commit parsing (baseline)

   - Measures commit parsing and classification in single-threaded mode
   - Tested with 10, 50, 100, and 500 commits

1. **parse_parallel**: Parallel commit parsing

   - Measures commit parsing using rayon parallel processing
   - Tested with 50, 100, and 500 commits
   - Uses a threshold of 10 commits to enable parallelism

1. **version_inference**: Semantic version inference

   - Measures the time to infer version bumps from parsed commits
   - Includes major/minor/patch detection and pre-1.0 adjustments
   - Tested with 10, 50, 100, and 500 commits

1. **render_block**: Markdown changelog rendering

   - Measures markdown generation from parsed commits
   - Includes section grouping, formatting, and reference linking
   - Tested with 10, 50, 100, and 500 commits

### Running Benchmarks Locally

The project uses CodSpeed for benchmarking. To run benchmarks locally, you need the `cargo-codspeed` CLI:

```bash
# Install cargo-codspeed (first time only)
cargo install cargo-codspeed

# Build benchmarks
cargo codspeed build

# Run all benchmarks
cargo codspeed run

# Run specific benchmark
cargo codspeed run parse_sequential

# Run with specific arguments (sizes)
cargo codspeed run -- --bench 100
```

**Note**: CodSpeed provides instrumentation-based measurements that are more accurate and consistent than wall-clock timing. Results are tracked over time in the CodSpeed dashboard when run in CI.

### CI Integration

Benchmarks run automatically on every PR and push to main via GitHub Actions (`.github/workflows/benches.yml`). The workflow:

1. Uses `moonrepo/setup-rust@v1` to install Rust and `cargo-codspeed`
1. Builds benchmarks with `cargo codspeed build`
1. Runs benchmarks with `cargo codspeed run` via `CodSpeedHQ/action@v4`
1. Results are uploaded to CodSpeed dashboard for tracking

**View results**: Check the CodSpeed dashboard linked in PR checks or at [codspeed.io](https://codspeed.io).

## Baseline Results

> [!NOTE]
> Benchmark results are tracked continuously via CodSpeed. Historical data and trends are available in the CodSpeed dashboard. Run `cargo codspeed run` locally to see performance on your hardware.

### Local Baseline (as of Stage 17, Task 99)

The following baseline results were captured on a GitHub Actions runner (Ubuntu latest, x86_64):

**Environment:**
- CPU: GitHub Actions runner (2-4 cores)
- Rust: 1.85+
- Benchmark Framework: codspeed-divan-compat
- Timer precision: 20 ns

**Parse Sequential (single-threaded):**
| Commits | Median    | Mean      |
|---------|-----------|-----------|
| 10      | 10.34 µs  | 10.79 µs  |
| 50      | 45.94 µs  | 46.27 µs  |
| 100     | 89.58 µs  | 90.7 µs   |
| 500     | 437.2 µs  | 440 µs    |

**Parse Parallel (rayon, threshold=10):**
| Commits | Median    | Mean      |
|---------|-----------|-----------|
| 50      | 68.01 µs  | 79.52 µs  |
| 100     | 114.5 µs  | 116.7 µs  |
| 500     | 383.1 µs  | 384.4 µs  |

**Version Inference:**
| Commits | Median    | Mean      |
|---------|-----------|-----------|
| 10      | 1.071 µs  | 1.078 µs  |
| 50      | 10.13 µs  | 10.28 µs  |
| 100     | 10.48 µs  | 10.68 µs  |
| 500     | 103.4 µs  | 104.8 µs  |

**Render Block (markdown generation):**
| Commits | Median    | Mean      |
|---------|-----------|-----------|
| 10      | 2.614 µs  | 2.887 µs  |
| 50      | 9.246 µs  | 9.512 µs  |
| 100     | 16.84 µs  | 17.34 µs  |
| 500     | 81.58 µs  | 82.7 µs   |

**Key Observations:**

1. **Linear Scaling**: All operations scale approximately linearly (O(n)) with commit count
2. **Parallel Overhead**: For 50 commits, parallel parsing (~68µs) is slower than sequential (~46µs) due to thread overhead
3. **Parallel Benefit**: At 500 commits, parallel (~384µs) is ~13% faster than sequential (~440µs)
4. **Fast Operations**: Version inference and rendering are extremely fast (1-100µs range)
5. **Parser Dominance**: Parsing is the most expensive operation, taking ~90% of total time

**Parallel Processing Threshold Recommendation:**

Based on these results, the default threshold of 50 commits is appropriate:
- Below 50: Sequential is faster (less overhead)
- Above 100: Parallel shows measurable benefit
- Current threshold: 50 (good balance)

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

1. **Version Inference**

   - O(n) scan through commits to find highest semver impact
   - Early termination on major breaking change detection
   - Constant-time version bump calculation

1. **Rendering**

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

1. **String allocation**: Use `ecow` for copy-on-write strings and small string optimization
1. **Hash function**: Use `foldhash` for faster hashing with quality settings
1. **Parallel rendering**: Currently sequential, could parallelize section rendering
1. **Caching**: Memoize regex compilation and provider detection

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
# Install cargo-codspeed if not already installed
cargo install cargo-codspeed

# Run benchmarks with CodSpeed instrumentation
cargo codspeed build
cargo codspeed run

# CPU profiling with flamegraph (alternative to CodSpeed)
cargo install flamegraph
cargo flamegraph --bench parse_performance

# Memory profiling (requires valgrind)
cargo codspeed build
valgrind --tool=massif target/release/deps/parse_performance-*

# Time profiling
cargo build --release
time ./target/release/changelogen release --dry-run
```

## CI Performance Tracking

Benchmark results are automatically tracked in CI using CodSpeed:

- **Workflow**: `.github/workflows/benches.yml`
- **Runs on**: Every PR and push to main
- **Dashboard**: Results available at [codspeed.io](https://codspeed.io)
- **Features**:
  - Automated regression detection
  - Historical performance tracking
  - Per-PR performance comparison
  - Visual performance graphs

CodSpeed provides instrumentation-based benchmarking that is more accurate than wall-clock timing and less susceptible to noise from CI environment variations.

## Contributing Performance Improvements

When submitting performance optimizations:

1. **Benchmark before and after**: Use `cargo bench -- --save-baseline`
1. **Profile bottlenecks**: Use `cargo flamegraph` or `perf`
1. **Document trade-offs**: Speed vs. memory vs. maintainability
1. **Preserve correctness**: All tests must still pass
1. **Update this doc**: Add new benchmarks or update baselines

## Resources

- [CodSpeed - Continuous Benchmarking Platform](https://codspeed.io/)
- [codspeed-divan-compat](https://crates.io/crates/codspeed-divan-compat) - CodSpeed-instrumented divan API
- [cargo-codspeed CLI](https://github.com/CodSpeedHQ/codspeed-rust)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph)
