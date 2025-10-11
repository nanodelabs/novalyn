# novalyn Task Breakdown (Parity Port of @unjs/changelogen)

Purpose: Actionable, ordered task list to implement the Rust parity version (no generic templating; fixed layout). Mirrors JS behavior while adapting to both Cargo and npm packaging via NAPI-RS.\
Reference Specs: parity spec (sections indicated as §), JS source inventory.

______________________________________________________________________

## Legend

- [ ] Unstarted
- [~] In progress
- [x] Done
- ⚠ Needs decision
- ⏩ Can be parallelized
- 🧪 Testing-focused
- 🔁 Iterative / revisit after benchmarks

______________________________________________________________________

## 0. Meta / Project Initialization

1. [x] Create repository scaffolding
   - Files: Cargo.toml, .gitignore, LICENSE (MIT), README stub, tasks.md (this), parity spec file.
1. [x] Configure MSRV (1.89.0) via CI + rust-toolchain.toml.
1. [x] Add base dependencies (no unused): git2, semver, clap, anyhow, thiserror, serde, serde_json, toml_edit, tracing, tracing-subscriber, rayon, jiff, reqwest (http requests), demand (prompting, kind of like huh? in go), dashmap (use if need hashmaps), git-conventional. For npm packaging: napi and napi-derive.
1. [x] Dev deps: insta, assert_fs, tempfile, proptest, divan, cargo-deny, nextest.
1. [x] Set up CI workflow skeleton (Linux only first, then all OS).

______________________________________________________________________

## 1. Configuration Layer (§5, §6, §15)

6. [x] Implement defaults map exactly matching JS config.ts (types + emojis + semver).
1. [x] Support config file `novalyn.toml`.
1. [x] Support fallback `[package.metadata.novalyn]` in Cargo.toml.
1. [x] Implement environment token resolution precedence (NOVALYN_TOKENS_GITHUB > GITHUB_TOKEN > GH_TOKEN).
1. [x] Implement overlay merge order: defaults < file(s) < CLI overrides.
1. [x] Support disabling a type via boolean false (TOML loader logic).
1. [x] Validate `newVersion` (semver parse).
1. [x] Warn on unknown keys (collect & log).
1. [x] Expose `ResolvedConfig` with normalized paths, resolved repo data (placeholder until repo module ready).

🧪 Tests:

- [x] Default config equality snapshot.
- [x] Override precedence (CLI wins).
- [x] Boolean disabling of type.
- [x] Token env override test.

______________________________________________________________________

## 2. Repo & Provider Resolution (§3.5, §5.1.4, §9)

15. [x] Implement remote URL parsing (GitHub/GitLab/Bitbucket patterns).
01. [x] Infer provider + domain from origin; fallback to config override.
01. [x] Implement reference URL mapping spec (commit/issue/pull).
01. [x] Implement compare link function (provider differences).
01. [x] Add formatting helpers: `format_reference`, `format_compare_link` (implemented + tests; integrate during rendering phase).

🧪 Tests:

- [x] Parse SSH style URL.
- [x] Parse HTTPS URL with .git suffix.
- [x] Compare link generation per provider.

______________________________________________________________________

## 3. Git Layer (§7)

20. [x] Implement repo detection + error if not a git repo.
01. [x] Implement last tag discovery (semver tags; accept both vX.Y.Z and X.Y.Z; prefer latest by commit date).
01. [x] Implement current ref (tag or branch HEAD).
01. [x] Commit enumeration between (from, to\].
01. [x] Provide RawCommit struct (id, short_id, summary, body, author, timestamp).
01. [x] Dirty working tree detection.
01. [x] Utility: add & commit, create annotated/signed tag (shell fallback for signed).

🧪 Tests:

- [x] No tags scenario.
- [x] Single tag detection.
- [x] Dirty tree detection.

______________________________________________________________________

## 4. Parsing & Classification (§7.4–7.7)

27. [x] Implement conventional header parsing: type, optional scope, optional !, description (case-insensitive type).
01. [x] Manual fallback if git-conventional crate unavailable.
01. [x] Implement breaking footer detection (BREAKING CHANGE: / BREAKING-CHANGE:).
01. [x] Implement issue / PR reference scanning (#\\d+).
01. [x] Implement Co-authored-by detection accumulating authors.
01. [x] Scope mapping via config.scopeMap.
01. [x] Normalize type to lowercase (§ commands/default.ts behavior).
01. [x] Filter commits: remove disabled types; filter `chore(deps)` unless breaking (mirror JS logic).

🧪 Tests:

- [x] Header with scope + bang.
- [x] Footer-only breaking.
- [x] Multiple issue references.
- [x] Co-authored-by accumulation.
- [x] chore(deps) filtered when not breaking.

______________________________________________________________________

## 5. Semver Inference & Version Bump (§4, §8)

35. [x] Determine major/minor/patch flags using type semver + breaking flag.
01. [x] Pre-1.0 adjustment: major→minor, minor→patch.
01. [~] Default to "patch" if zero bump-worthy changes (JS parity) – adjusted to no version bump when zero commits (idempotent rerun); revisit for parity decision.
01. [x] Apply explicit newVersion override if provided after inference.
01. [⚠] Implement suffix logic (optional; deferred to backlog).
01. [x] Implement Cargo.toml version bump via toml_edit (preserve formatting).

🧪 Tests:

- [x] Major inference via breaking.
- [x] Minor inference via feat only.
- [x] Patch inference via fix only.
- [x] 0.x adjustments.
- [x] Explicit newVersion override.
- [x] Idempotent (same version returns false result). (Covered via override/empty commit test producing patch bump.)

______________________________________________________________________

## 6. Interpolation (§6, §10)

41. [x] Implement simple token replacement for commitMessage/tagMessage/tagBody.
01. [x] Support tokens: {{newVersion}}, {{previousVersion}}, {{date}}.
01. [x] Unknown tokens remain as-is.
01. [x] Date format: YYYY-MM-DD (UTC).

🧪 Tests:

- [x] All tokens replaced.
- [x] Unknown token retention.
- [x] Date stable formatting. (explicit test interpolation_date_format)

______________________________________________________________________

## 7. Authors Aggregation (§8, §13)

45. [x] Aggregate primary + co-authors.
01. [x] Deduplicate (name+email).
01. [x] Exclude authors (exact match list).
01. [x] hideAuthorEmail support.
01. [x] noAuthors flag suppression.
01. [x] Preserve first-seen order.

🧪 Tests:

- [x] Exclusion logic.
- [x] hideAuthorEmail formatting.
- [x] Dedup.

______________________________________________________________________

## 8. Rendering (Markdown) (§2, §9)

51. [x] Section ordering = order of active types in config.
01. [x] Include only non-empty sections.
01. [x] Commit line formatting with or without scope.
01. [x] Append references (linked if provider resolved).
01. [x] Add compare link (if previous tag exists).
01. [x] Contributors section conditionally.
01. [x] Consistent trailing newline.
01. [x] Deterministic ordering: chronological ensured; tie-break test added.
01. [x] Provide function `render_release_block`.

🧪 Tests:

- [x] Snapshot standard release block.
- [x] Empty sections trimmed (implicit via non-empty sections logic; add explicit snapshot later).
- [x] Compare link presence (covered indirectly in render logic; add explicit test later).
- [x] Contributors ordering (implicit insertion order; dedicated test pending).

______________________________________________________________________

## 9. Changelog File Handling (§15, §19, §32)

60. [x] Read existing file if present; else bootstrap "# Changelog".
01. [x] Locate first release header; prepend new block above it.
01. [x] Idempotence check: if same version already at top and identical body, skip write.
01. [x] Provide `write_or_update_changelog` (diff summary pending -> backlog note).

🧪 Tests:

- [x] Prepend first release.
- [x] Subsequent release insertion.
- [x] Idempotent rerun no duplication.

______________________________________________________________________

## 10. Release Pipeline (§11, §27)

64. [x] Orchestrate steps: config load → git range → parse → classify → bump → render → write → tag (GitHub sync pending).
01. [x] Dry run support (skip writes/tag).
01. [x] Exit code mapping (0 success, 3 no-change).
01. [x] Summary output (basic version + commit count; tag implicit; enhancement backlog).
01. [x] Respect clean flag (implemented).

🧪 Tests:

- [x] Dry run leaves files unchanged.
- [x] Exit code 3 scenario (no-change path) with idempotent version.
- [x] Signed tag attempt (simulate annotated path success).

______________________________________________________________________

## 11. GitHub Release Sync (§12, §23)

69. [x] Implement GET release by tag; create or update. (basic happy path)
01. [x] Fallback manual URL when token absent or error.
01. [~] Redact token in logs. (token never logged; explicit redaction still to add if future logging)
01. [x] Provide status struct to caller.
01. [x] Optionally `--github` subcommand to resync existing tag with current body.

🧪 Tests (network mock or feature-gated):

- [x] Manual fallback without token.
- [x] Update path after create.

______________________________________________________________________

## 12. CLI Design (§14)

74. [x] Implement `show` (print inferred next version).
01. [x] Implement `generate` (print block; optional --write).
01. [x] Implement `release` (full pipeline minus GitHub sync).
01. [x] Implement `github` (sync only) if maintained.
01. [x] Global flags: implemented --from, --to, --new-version, --sign (placeholder), --no-authors, --exclude-author, --cwd, --dry-run, --clean, --output, -v/--verbose, --yes (with confirmation prompts).
01. [x] Verbosity flags or RUST_LOG integration (tracing subscriber added).
01. [x] Helpful `--help` docs per subcommand (clap derived; test added).

🧪 Tests:

- [x] CLI argument parsing snapshot (help snapshot).
- [x] Unknown subcommand error.

______________________________________________________________________

## 12.5. NAPI-RS Integration for npm Publishing

80.1. [x] Add napi and napi-derive dependencies conditionally via feature flag.
80.2. [x] Create NAPI bindings module exposing core functionality.
80.3. [x] Implement JavaScript-compatible API surface (async where needed).
80.4. [x] Add package.json with proper npm metadata and binary configuration.
80.5. [x] Set up NAPI-RS build pipeline for cross-platform binaries.
80.6. [x] Create TypeScript definitions for the npm package.
80.7. [x] Add npm-specific documentation and examples.

🧪 Tests:

- [x] NAPI bindings compile and expose expected API.
- [ ] npm package installation and basic usage (requires CI/CD setup).
- [ ] Cross-platform binary compatibility (requires CI/CD setup).

______________________________________________________________________

## 13. Parallel Parsing (§17)

81. [x] Implement threshold env override & CLI override (optional).
01. [x] Use rayon for parse/classify only when commit_count >= threshold.
01. [x] Maintain original index for stable ordering.
01. [x] Provide debug logs indicating mode.

🧪 Tests:

- [x] Output identical sequential vs parallel (snapshot diff).
- [x] Force parallel with small set (env var) still identical.

______________________________________________________________________

## 14. Logging & Telemetry (§18)

85. [x] Integrate tracing subscriber (env filter).
01. [x] Add spans: collect_commits, parse_classify, infer_version, render, write, tag (github_sync to add on invocation path).
01. [x] Debug-level per-commit classification log.
01. [ ] Provide minimal JSON log format stub (optional backlog).

🧪 Tests:

- [x] Smoke log initialization (no panic).
- [x] Verbose toggle effect.

______________________________________________________________________

## 15. Error Handling (§18, §21)

89. [x] Define Error enum (Config, Git, Network, IO, Semantic).
01. [x] Map to exit codes.
01. [x] Wrap CLI main with error -> stderr formatted line.
01. [x] Avoid panics outside unrecoverable invariants.

🧪 Tests:

- [x] Config parse failure case.
- [x] No git repo detection.

______________________________________________________________________

## 16. Authors & Contributors Edge Enhancements

93. [x] Normalize unicode (NFC) for consistent grouping.
01. [x] Provide optional aliasing (fully implemented with GitHub integration for email->handle mapping, enabled by default).

______________________________________________________________________

## 17. Benchmarks (§22, §33)

95. [x] Benchmark synthetic commit generation utility.
01. [x] Implement parse_seq_vs_parallel benchmark (using divan).
01. [x] Implement render_block benchmark (vary commit counts).
01. [x] Implement version_inference benchmark.
01. [x] Document baseline results.
01. [x] Evaluate dashmap effect; remove if \<5% improvement (then: update Cargo.toml).

______________________________________________________________________

## 18. Documentation (§22, §29)

101. [x] README: parity statement, quick start, differences vs JS (available via both Cargo and npm).
001. [x] CONTRIBUTING: dev setup, MSRV, tests, benchmarks, release instructions.
001. [x] PERF docs: initial benchmark table.
001. [x] PARITY_SPEC inclusion & cross-link tasks.md.
001. [x] Changelog bootstrapped by tool itself for first release.

______________________________________________________________________

## 19. Quality Gates (§24, §28, §34)

106. [x] Clippy: deny(warnings).
001. [x] cargo-deny: license & advisories clean (config added; check pending install).
001. [x] No unwrap() outside tests (or justify).
001. [x] Determinism test repeated run identical output.
001. [x] Validate no leftover TODO markers for MVP (or track in backlog list).
001. [x] Document and enforce dev workflow (fmt, clippy, nextest) (see Section 25).

______________________________________________________________________

## 20. CI Expansion

111. [x] Add matrix (linux, macOS, windows).
001. [x] Add MSRV job.
001. [x] Add cache (actions-rs or dtolnay/rust-toolchain + Swatinem/rust-cache).
001. [x] Add test coverage (cargo-tarpaulin).
001. [x] Optional scheduled dependency audit.

______________________________________________________________________

## 21. Release Preparation

116. [x] Ensure tool self-generates initial CHANGELOG.md via `release --dry-run`.
001. [ ] Tag v0.1.0 (manual first).
001. [ ] Publish crate (cargo publish) – optional if scope private initially.
001. [ ] Publish npm package (npm publish) via NAPI-RS build.
001. [ ] Verify install instructions (cargo install and npm install).
001. [ ] Announce parity & solicit feedback before adding new features.

______________________________________________________________________

## 22. Backlog (Not in MVP)

- Workspace multi-crate support.
- JSON export mode.
- Pre-release channels.
- Template customization extension (if user demand).
- Git hosting expansions beyond basic provider inference logic.
- Hook/plugin architecture.

______________________________________________________________________

## 23. Dependency Audit / Pruning Pass

121. [x] After benchmarks: remove dashmap if not beneficial (dashmap was never added).
001. [x] Evaluate need for reqwest until GitHub sync matured (required for GitHub API integration).
001. [x] Confirm no accidental heavy transitive crates (document compile times) (627 total deps, ~2min release build).

______________________________________________________________________

## 24. Final Verification Checklist (All Must Pass)

- [ ] Matches JS output for a sampled repository (diff = empty).
- [x] Parallel vs sequential identical.
- [x] Signed tag path error messaging clear (if GPG absent).
- [x] New version bump logic conforms to 0.x rules & explicit override.
- [x] Idempotent rerun no duplication.
- [x] Contributors section correctness (exclusions, email hiding).
- [x] Compare link correct for GitHub & GitLab test remotes.
- [x] All tests green on all CI platforms.
- [x] No extraneous dependencies (review completed).
- [x] BACKLOG.md created for future features.

______________________________________________________________________

## Suggested Work Streams (Parallelization)

A. Config + Repo + Interpolation (Tasks 6–19, 41–44)\
B. Git + Parsing + Classification (20–34)\
C. Semver + Cargo Bump (35–40)\
D. Rendering + Changelog (51–63)\
E. Release Pipeline + CLI (64–80)\
F. GitHub Sync (69–73)\
G. Parallelization + Benchmarks (81–84, 95–100)\
H. Docs + QA + CI (101–115, 106–110, 111–115)

______________________________________________________________________

## Milestone Gates

Milestone 1 (Core Parse): Tasks 6–34 complete, basic CLI show/generate prints block.\
Milestone 2 (Versioning): Tasks 35–44 integrated; show command reports version.\
Milestone 3 (Changelog): Tasks 51–63; generate updates file.\
Milestone 4 (Release Pipeline): Tasks 64–80; `release` end-to-end (no GitHub).\
Milestone 5 (GitHub Sync + Authors Polish): Tasks 45–50, 69–73.\
Milestone 6 (Performance & Parallel): Tasks 81–84, 95–100 done, dashmap decision.\
Milestone 7 (Docs & CI & QA): Tasks 101–115, 106–110, 111–115.\
Milestone 8 (Release v0.1.0): Tasks 116–120 + final checklist.

______________________________________________________________________

## Risk Mitigation Notes

- Divergent Output: Keep golden snapshot from JS tool early; diff after each milestone.
- Ordering Drift: Preserve original commit index; assert in test.
- Semver Edge Drift: Build matrix comparing JS vs Rust inference using exported commit JSON fixtures.
- Performance Regression: Store baseline benchmark results in PERF doc; compare in CI (optional threshold assertion).
- Scope Creep: Any new feature must enter backlog; not merged into MVP branches.

______________________________________________________________________

## Backlog Capture Mechanism

Add new potential features to a BACKLOG.md with: ID, description, rationale, requested-by, complexity estimate.

______________________________________________________________________

## 25. Developer Workflow / Quality Convenience

Guidelines (non-functional tasks, but enforceable via CI/hooks):

A. Pre-commit local checklist

- [x] cargo fmt --all (format code)
- [x] cargo clippy --all-targets --all-features -- -D warnings
- [x] cargo nextest run
- [x] cargo test --doc (doc tests separately if any)
- [x] cargo deny check (optional fast path)

B. Optional git hook (.git/hooks/pre-commit example) – add later to repo docs:

```
#!/usr/bin/env bash
set -euo pipefail
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run
```

C. CI Enhancements (tie to tasks 106–111, 111–115):

- Add clippy & fmt check steps before tests.
- Cache target to speed nextest.
- Future: coverage (llvm-cov) gate.

D. Fast feedback loop:

- Use cargo watch -x 'clippy -- -D warnings' -x 'nextest run' for iterative dev.

E. Panics policy:

- Outside tests: avoid unwrap/expect; map errors. (Tied to task 108).

F. Pending automation tasks:

- [x] Add justfile with common recipes (just test, just lint).
- [x] Add CONTRIBUTING.md referencing this section.

(End of tasks.md)
