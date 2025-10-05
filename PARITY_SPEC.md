# Parity Spec: novalyn vs @unjs/changelogen

**Goal**: Achieve output parity (format, ordering, inference) with the JS version while adapting to Rust tooling (Cargo, git2) and keeping a fixed template (no user templating in MVP).

See also: [tasks.md](tasks.md) for detailed implementation checklist.

______________________________________________________________________

## Scope (MVP)

### In Scope

- ✅ Config surface identical where meaningful (types, emojis, semver directives, token precedence)
- ✅ Commit classification logic mirrors JS (conventional commits + breaking detection + filtering rules)
- ✅ Version inference rules including 0.x adjustments
- ✅ Markdown release block layout + compare links + contributors list
- ✅ Changelog insertion & idempotence
- ✅ Release pipeline orchestration excluding advanced GitHub interactions (basic sync only)

### Non-Goals (Backlog)

- ⏸ Workspace / multi-crate aggregation
- ⏸ Arbitrary template customization
- ⏸ Extended hosting providers beyond GitHub/GitLab/Bitbucket baseline
- ⏸ Advanced GitHub release features (draft releases, release assets)

______________________________________________________________________

## Configuration Parity

### Default Commit Types

Both implementations use identical default types with same ordering:

| Type       | Title                  | Emoji | Semver Impact |
| ---------- | ---------------------- | ----- | ------------- |
| `feat`     | Features               | ✨    | minor         |
| `fix`      | Bug Fixes              | 🐞    | patch         |
| `perf`     | Performance            | ⚡️    | patch         |
| `docs`     | Documentation          | 📚    | none          |
| `refactor` | Refactors              | 🛠     | patch         |
| `style`    | Styles                 | 🎨    | none          |
| `test`     | Tests                  | 🧪    | none          |
| `build`    | Build System           | 📦    | none          |
| `ci`       | Continuous Integration | 👷    | none          |
| `chore`    | Chores                 | 🧹    | none          |
| `revert`   | Reverts                | ⏪    | patch         |

**Implementation**: `src/config.rs::default_types()`

### Configuration Sources & Precedence

Identical precedence order:

1. **CLI arguments** (highest priority)
1. **novalyn.toml** in project root
1. **Cargo.toml** `[package.metadata.novalyn]` section (Rust) / **package.json** (JS)
1. **Defaults** (lowest priority)

**Rust specifics**:

- Uses TOML instead of JSON/JS for configuration files
- Supports both `novalyn.toml` and `Cargo.toml` embedding

### Environment Variables

Token resolution precedence (identical):

1. `NOVALYN_TOKENS_GITHUB`
1. `GITHUB_TOKEN`
1. `GH_TOKEN`

**Additional Rust variable**:

- `NOVALYN_PARALLEL_THRESHOLD`: Control parallel parsing (default: 50)

______________________________________________________________________

## Commit Parsing Parity

### Conventional Commit Format

Identical parsing logic:

```
<type>[optional scope][optional !]: <description>

[optional body]

[optional footer(s)]
```

**Case handling**: Type is normalized to lowercase (e.g., `FEAT` → `feat`)

### Breaking Change Detection

Three ways to mark breaking changes (all supported):

1. **Exclamation mark**: `feat!: breaking change`
1. **Footer with colon**: `BREAKING CHANGE: description`
1. **Footer with dash**: `BREAKING-CHANGE: description`

### Issue/PR References

Identical pattern matching:

- `#123` → Issue/PR 123
- Multiple refs supported: `fixes #123, closes #456`
- Extracted and linked in output

### Co-authored-by Detection

Footer pattern (case-insensitive):

```
Co-authored-by: Name <email@example.com>
```

**Behavior**: All co-authors accumulated and included in contributors list

### Scope Mapping

`scopeMap` configuration applies transformations:

```toml
[scopeMap]
"deps-dev" = ""        # Remove scope
"ui" = "frontend"      # Rename scope
```

______________________________________________________________________

## Filtering Rules

### Disabled Types

Types can be disabled via configuration:

```toml
[types.docs]
enabled = false
```

**Effect**: Commits of disabled types are excluded from changelog

### chore(deps) Special Case

Behavior (identical to JS):

- `chore(deps): ...` commits **excluded** by default
- `chore(deps)!: ...` **included** (breaking change overrides filter)

**Rationale**: Reduce noise from automated dependency updates

______________________________________________________________________

## Version Inference Parity

### Semver Rules

Identical semver impact calculation:

| Condition                                   | Impact |
| ------------------------------------------- | ------ |
| Breaking change (`!` or `BREAKING CHANGE:`) | major  |
| feat commit                                 | minor  |
| fix/perf commit                             | patch  |
| docs/style/test/ci/build commit             | none   |

### Pre-1.0 Adjustment

For versions where `major == 0`:

| JS Impact | Version 0.x Impact |
| --------- | ------------------ |
| major →   | minor              |
| minor →   | patch              |
| patch →   | patch              |

**Example**: `0.5.0` with breaking change → `0.6.0` (not `1.0.0`)

### Default Bump Behavior

| Scenario                                    | JS Version   | Rust Version | Status       |
| ------------------------------------------- | ------------ | ------------ | ------------ |
| No commits at all                           | No change    | No change    | ✅ Identical |
| Commits with all `none` impact (e.g., docs) | Patch bump   | Patch bump   | ✅ Identical |
| Override version provided                   | Use override | Use override | ✅ Identical |

**Implementation**: `src/parse.rs::infer_version()`

**Note**: Task 37 tracks final parity verification for zero-impact scenarios.

### Explicit Version Override

CLI flag `--new-version X.Y.Z` or config `newVersion = "X.Y.Z"`:

**Behavior**: Bypasses inference, uses exact version specified

______________________________________________________________________

## Markdown Output Parity

### Release Block Structure

Identical format:

```markdown
## v1.2.3

[compare changes](https://github.com/org/repo/compare/v1.2.2...v1.2.3)

### 🚀 Features

- **scope**: commit description (#123)
- another feature

### 🐛 Bug Fixes

- fix something (#456, #789)

### ❤️ Contributors

- Alice Smith
- Bob Jones <bob@example.com>
```

### Section Ordering

Sections appear in the order defined by `types` configuration (default order preserved).

**Empty sections**: Omitted (no empty section headers)

### Commit Formatting

```markdown
- **scope**: description (#123)  // with scope
- description (#456)              // without scope
```

**Reference links**:

- GitHub: `#123` → `[#123](https://github.com/org/repo/issues/123)`
- GitLab: `#123` → `[#123](https://gitlab.com/org/repo/-/issues/123)`
- Bitbucket: `#123` → `[#123](https://bitbucket.org/org/repo/issues/123)`

### Compare Link

Format: `[compare changes](PROVIDER_COMPARE_URL)`

**Providers**:

- GitHub: `/compare/v1.2.2...v1.2.3`
- GitLab: `/-/compare/v1.2.2...v1.2.3`
- Bitbucket: `/branches/compare/v1.2.3..v1.2.2` (reversed!)

### Contributors Section

**Formatting**:

- `hideAuthorEmail: false` → `Name <email@example.com>`
- `hideAuthorEmail: true` → `Name`
- `noAuthors: true` → Section omitted entirely

**Ordering**: First-seen order (deterministic)

**Deduplication**: By name+email pair

______________________________________________________________________

## Changelog File Operations

### File Structure

```markdown
# Changelog

## v1.2.3

...

## v1.2.2

...
```

### Insertion Logic

1. **Bootstrap**: If file missing, create with `# Changelog` header
1. **Prepend**: Insert new release block after main header, before first existing release
1. **Idempotence**: If same version already at top with identical content, skip write

**Implementation**: `src/changelog.rs`

______________________________________________________________________

## Git Operations Parity

### Tag Discovery

**Logic** (identical):

1. Find all tags matching semver pattern (`vX.Y.Z` or `X.Y.Z`)
1. Sort by commit date (most recent first)
1. Return latest

**Special cases**:

- No tags → No previous version (treat as initial release)
- Invalid semver tags → Ignored

### Commit Range

**Range**: `(previous_tag, HEAD]` (exclusive start, inclusive end)

**Special case**: If no previous tag, include all commits up to HEAD

______________________________________________________________________

## Differences from JS Version

### Intentional Differences

| Feature          | JS Version                       | Rust Version                       | Rationale                |
| ---------------- | -------------------------------- | ---------------------------------- | ------------------------ |
| Config format    | JS/JSON                          | TOML                               | Rust ecosystem standard  |
| Config file      | `package.json` or `.changelogrc` | `Cargo.toml` or `novalyn.toml` | Cargo integration        |
| Parallel parsing | Not available                    | Available (rayon)                  | Performance optimization |
| Package manager  | npm                              | Cargo (+ npm via NAPI-RS planned)  | Native Rust tooling      |

### Must-Have Parity

These behaviors **must** match JS version exactly:

- ✅ Commit classification (type, scope, breaking)
- ✅ Version inference (semver + pre-1.0 rules)
- ✅ Markdown output format
- ✅ Filtering rules (disabled types, chore(deps))
- ✅ Contributor aggregation
- ✅ Idempotent changelog updates

### Nice-to-Have Parity

These behaviors **should** match but minor deviations acceptable:

- ⚠️ Error messages (must be helpful but wording can differ)
- ⚠️ CLI flag naming (prefer Rust conventions but align where possible)
- ⚠️ Log output format (use tracing instead of console.log)

______________________________________________________________________

## Verification Strategy

### Golden Snapshot Testing

1. **Create test repository** with known commit history
1. **Run JS version**: Capture CHANGELOG.md output
1. **Run Rust version**: Capture CHANGELOG.md output
1. **Diff outputs**: Should be byte-identical (modulo intentional differences)

**Test files**: `tests/render_block_snapshot.rs`, `tests/render_snapshot.rs`

### Property-Based Testing

Use `proptest` to verify invariants:

- Parsing is deterministic (same input → same output)
- Version inference is monotonic (never downgrades)
- Markdown output is valid
- References are properly escaped

**Implementation**: Deferred to post-MVP (see tasks.md)

### Benchmark Comparison

**Goal**: No more than 10% performance regression vs JS version

**Metrics**:

- Commits per second (parsing)
- End-to-end time (release command)
- Memory usage

**Baseline**: To be established after initial release (see PERF.md)

### CI Validation

All parity-critical tests run on every commit:

- Unit tests (parsing, classification, inference)
- Integration tests (full pipeline)
- Snapshot tests (markdown output)
- Determinism tests (repeated runs)

**Status**: ✅ Implemented in `.github/workflows/ci.yml`

______________________________________________________________________

## Known Deviations

### Documented Differences

None currently. Any deviation discovered must be:

1. **Documented** in this section
1. **Justified** with technical rationale
1. **Approved** via issue discussion
1. **Noted** in README.md differences section

### Intentional Future Deviations

Planned features that diverge from JS version:

- **Workspace support**: Multi-crate changelog aggregation (Rust-specific)
- **Cargo integration**: Automatic version bumping in Cargo.toml
- **Static binary**: Easier distribution without Node.js runtime

______________________________________________________________________

## Cross-References

- **Task breakdown**: [tasks.md](tasks.md)
- **Development guide**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Performance tracking**: [PERF.md](PERF.md)
- **Agent instructions**: [AGENTS.md](AGENTS.md)

______________________________________________________________________

## Maintenance Notes

This spec should be updated when:

- ✅ New parity test added → Document coverage
- ✅ Deviation discovered → Add to Known Deviations
- ✅ Feature completed → Mark scope items complete
- ✅ JS version changes → Review and update mappings

**Last Updated**: 2024-01-01 (Initial expansion from stub)
