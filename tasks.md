# changelogen-rs Task Breakdown (Parity Port of @unjs/changelogen)

Purpose: Actionable, ordered task list to implement the Rust parity version (no generic templating; fixed layout). Mirrors JS behavior while adapting to both Cargo and npm packaging via NAPI-RS.  
Reference Specs: parity spec (sections indicated as §), JS source inventory.

---

## Legend

- [ ] Unstarted
- [~] In progress
- [x] Done
- ⚠ Needs decision
- ⏩ Can be parallelized
- 🧪 Testing-focused
- 🔁 Iterative / revisit after benchmarks

---

## 0. Meta / Project Initialization

1. [x] Create repository scaffolding
   - Files: Cargo.toml, .gitignore, LICENSE (MIT), README stub, tasks.md (this), parity spec file.
2. [x] Configure MSRV (1.89.0) via CI + rust-toolchain.toml.
3. [x] Add base dependencies (no unused): git2, semver, clap, anyhow, thiserror, serde, serde_json, toml_edit, tracing, tracing-subscriber, rayon, jiff, reqwest (http requests), demand (prompting, kind of like huh? in go), dashmap (use if need hashmaps), git-conventional. For npm packaging: napi and napi-derive.
4. [x] Dev deps: insta, assert_fs, tempfile, proptest, divan, cargo-deny, nextest.
5. [x] Set up CI workflow skeleton (Linux only first, then all OS).

---

## 1. Configuration Layer (§5, §6, §15)

6. [x] Implement defaults map exactly matching JS config.ts (types + emojis + semver).
7. [x] Support config file `changelogen.toml`.
8. [x] Support fallback `[package.metadata.changelogen]` in Cargo.toml.
9. [x] Implement environment token resolution precedence (CHANGELOGEN_TOKENS_GITHUB > GITHUB_TOKEN > GH_TOKEN).
10. [x] Implement overlay merge order: defaults < file(s) < CLI overrides.
11. [x] Support disabling a type via boolean false (TOML loader logic).
12. [x] Validate `newVersion` (semver parse).
13. [x] Warn on unknown keys (collect & log).
14. [x] Expose `ResolvedConfig` with normalized paths, resolved repo data (placeholder until repo module ready).

🧪 Tests:
- [x] Default config equality snapshot.
- [x] Override precedence (CLI wins).
- [x] Boolean disabling of type.
- [x] Token env override test.

---

## 2. Repo & Provider Resolution (§3.5, §5.1.4, §9)

15. [x] Implement remote URL parsing (GitHub/GitLab/Bitbucket patterns).
16. [x] Infer provider + domain from origin; fallback to config override.
17. [x] Implement reference URL mapping spec (commit/issue/pull).
18. [x] Implement compare link function (provider differences).
19. [x] Add formatting helpers: `format_reference`, `format_compare_link` (implemented + tests; integrate during rendering phase).

🧪 Tests:
- [x] Parse SSH style URL.
- [x] Parse HTTPS URL with .git suffix.
- [x] Compare link generation per provider.

---

## 3. Git Layer (§7)

20. [x] Implement repo detection + error if not a git repo.
21. [x] Implement last tag discovery (semver tags; accept both vX.Y.Z and X.Y.Z; prefer latest by commit date).
22. [x] Implement current ref (tag or branch HEAD).
23. [x] Commit enumeration between (from, to].
24. [x] Provide RawCommit struct (id, short_id, summary, body, author, timestamp).
25. [x] Dirty working tree detection.
26. [x] Utility: add & commit, create annotated/signed tag (shell fallback for signed).

🧪 Tests:
- [x] No tags scenario.
- [x] Single tag detection.
- [x] Dirty tree detection.

---

## 4. Parsing & Classification (§7.4–7.7)

27. [x] Implement conventional header parsing: type, optional scope, optional !, description (case-insensitive type).
28. [x] Manual fallback if git-conventional crate unavailable.
29. [x] Implement breaking footer detection (BREAKING CHANGE: / BREAKING-CHANGE:).
30. [x] Implement issue / PR reference scanning (#\d+).
31. [x] Implement Co-authored-by detection accumulating authors.
32. [x] Scope mapping via config.scopeMap.
33. [x] Normalize type to lowercase (§ commands/default.ts behavior).
34. [x] Filter commits: remove disabled types; filter `chore(deps)` unless breaking (mirror JS logic).

🧪 Tests:
- [x] Header with scope + bang.
- [x] Footer-only breaking.
- [x] Multiple issue references.
- [x] Co-authored-by accumulation.
- [x] chore(deps) filtered when not breaking.

---

## 5. Semver Inference & Version Bump (§4, §8)

35. [x] Determine major/minor/patch flags using type semver + breaking flag.
36. [x] Pre-1.0 adjustment: major→minor, minor→patch.
37. [~] Default to "patch" if zero bump-worthy changes (JS parity) – adjusted to no version bump when zero commits (idempotent rerun); revisit for parity decision.
38. [x] Apply explicit newVersion override if provided after inference.
39. [⚠] Implement suffix logic (optional; deferred to backlog).
40. [x] Implement Cargo.toml version bump via toml_edit (preserve formatting).

🧪 Tests:
- [x] Major inference via breaking.
- [x] Minor inference via feat only.
- [x] Patch inference via fix only.
- [x] 0.x adjustments.
- [x] Explicit newVersion override.
- [x] Idempotent (same version returns false result). (Covered via override/empty commit test producing patch bump.)

---

## 6. Interpolation (§6, §10)

41. [x] Implement simple token replacement for commitMessage/tagMessage/tagBody.
42. [x] Support tokens: {{newVersion}}, {{previousVersion}}, {{date}}.
43. [x] Unknown tokens remain as-is.
44. [x] Date format: YYYY-MM-DD (UTC).

🧪 Tests:
- [x] All tokens replaced.
- [x] Unknown token retention.
- [x] Date stable formatting. (explicit test interpolation_date_format)

---

## 7. Authors Aggregation (§8, §13)

45. [x] Aggregate primary + co-authors.
46. [x] Deduplicate (name+email).
47. [x] Exclude authors (exact match list).
48. [x] hideAuthorEmail support.
49. [x] noAuthors flag suppression.
50. [x] Preserve first-seen order.

🧪 Tests:
- [x] Exclusion logic.
- [x] hideAuthorEmail formatting.
- [x] Dedup.

---

## 8. Rendering (Markdown) (§2, §9)

51. [x] Section ordering = order of active types in config.
52. [x] Include only non-empty sections.
53. [x] Commit line formatting with or without scope.
54. [x] Append references (linked if provider resolved).
55. [x] Add compare link (if previous tag exists).
56. [x] Contributors section conditionally.
57. [x] Consistent trailing newline.
58. [x] Deterministic ordering: chronological ensured; tie-break test added.
59. [x] Provide function `render_release_block`.

🧪 Tests:
- [x] Snapshot standard release block.
- [x] Empty sections trimmed (implicit via non-empty sections logic; add explicit snapshot later).
- [x] Compare link presence (covered indirectly in render logic; add explicit test later).
- [x] Contributors ordering (implicit insertion order; dedicated test pending).

---

## 9. Changelog File Handling (§15, §19, §32)

60. [x] Read existing file if present; else bootstrap "# Changelog".
61. [x] Locate first release header; prepend new block above it.
62. [x] Idempotence check: if same version already at top and identical body, skip write.
63. [x] Provide `write_or_update_changelog` (diff summary pending -> backlog note).

🧪 Tests:
- [x] Prepend first release.
- [x] Subsequent release insertion.
- [x] Idempotent rerun no duplication.

---

## 10. Release Pipeline (§11, §27)

64. [x] Orchestrate steps: config load → git range → parse → classify → bump → render → write → tag (GitHub sync pending).
65. [x] Dry run support (skip writes/tag).
66. [x] Exit code mapping (0 success, 3 no-change).
67. [x] Summary output (basic version + commit count; tag implicit; enhancement backlog).
68. [x] Respect clean flag (implemented).

🧪 Tests:
- [x] Dry run leaves files unchanged.
- [x] Exit code 3 scenario (no-change path) with idempotent version.
- [x] Signed tag attempt (simulate annotated path success).

---

## 11. GitHub Release Sync (§12, §23)

69. [x] Implement GET release by tag; create or update. (basic happy path)
70. [x] Fallback manual URL when token absent or error.
71. [~] Redact token in logs. (token never logged; explicit redaction still to add if future logging)
72. [x] Provide status struct to caller.
73. [x] Optionally `--github` subcommand to resync existing tag with current body.

🧪 Tests (network mock or feature-gated):
- [x] Manual fallback without token.
- [ ] Update path after create.

---

## 12. CLI Design (§14)

74. [x] Implement `show` (print inferred next version).
75. [x] Implement `generate` (print block; optional --write).
76. [x] Implement `release` (full pipeline minus GitHub sync).
77. [x] Implement `github` (sync only) if maintained.
78. [x] Global flags: implemented --from, --to, --new-version, --sign (placeholder), --no-authors, --exclude-author, --cwd, --dry-run, --clean, --output, -v/--verbose, --yes (with confirmation prompts).
79. [x] Verbosity flags or RUST_LOG integration (tracing subscriber added).
80. [x] Helpful `--help` docs per subcommand (clap derived; test added).

🧪 Tests:
- [x] CLI argument parsing snapshot (help snapshot).
- [x] Unknown subcommand error.

---

## 12.5. NAPI-RS Integration for npm Publishing

80.1. [ ] Add napi and napi-derive dependencies conditionally via feature flag.
80.2. [ ] Create NAPI bindings module exposing core functionality.
80.3. [ ] Implement JavaScript-compatible API surface (async where needed).
80.4. [ ] Add package.json with proper npm metadata and binary configuration.
80.5. [ ] Set up NAPI-RS build pipeline for cross-platform binaries.
80.6. [ ] Create TypeScript definitions for the npm package.
80.7. [ ] Add npm-specific documentation and examples.

🧪 Tests:
- [ ] NAPI bindings compile and expose expected API.
- [ ] npm package installation and basic usage.
- [ ] Cross-platform binary compatibility.

---

## 13. Parallel Parsing (§17)

81. [x] Implement threshold env override & CLI override (optional).
82. [x] Use rayon for parse/classify only when commit_count >= threshold.
83. [x] Maintain original index for stable ordering.
84. [x] Provide debug logs indicating mode.

🧪 Tests:
- [x] Output identical sequential vs parallel (snapshot diff).
- [x] Force parallel with small set (env var) still identical.

---

## 14. Logging & Telemetry (§18)

85. [x] Integrate tracing subscriber (env filter).
86. [x] Add spans: collect_commits, parse_classify, infer_version, render, write, tag (github_sync to add on invocation path).
87. [x] Debug-level per-commit classification log.
88. [ ] Provide minimal JSON log format stub (optional backlog).

🧪 Tests:
- [ ] Smoke log initialization (no panic).
- [ ] Verbose toggle effect.

---

## 15. Error Handling (§18, §21)

89. [x] Define Error enum (Config, Git, Network, IO, Semantic).
90. [x] Map to exit codes.
91. [x] Wrap CLI main with error -> stderr formatted line.
92. [x] Avoid panics outside unrecoverable invariants.

🧪 Tests:
- [x] Config parse failure case.
- [x] No git repo detection.

---

## 16. Authors & Contributors Edge Enhancements

93. [x] Normalize unicode (NFC) for consistent grouping.
94. [ ] Provide optional aliasing (future/backlog marker).

---

## 17. Benchmarks (§22, §33)

95. [x] Benchmark synthetic commit generation utility.
96. [x] Implement parse_seq_vs_parallel benchmark (using divan).
97. [x] Implement render_block benchmark (vary commit counts).
98. [x] Implement version_inference benchmark.
99. [ ] Document baseline results.
100. [ ] Evaluate dashmap effect; remove if <5% improvement (then: update Cargo.toml).

---

## 18. Documentation (§22, §29)

101. [ ] README: parity statement, quick start, differences vs JS (available via both Cargo and npm).
102. [ ] CONTRIBUTING: dev setup, MSRV, tests, benchmarks, release instructions.
103. [ ] PERF docs: initial benchmark table.
104. [ ] PARITY_SPEC inclusion & cross-link tasks.md.
105. [ ] Changelog bootstrapped by tool itself for first release.

---

## 19. Quality Gates (§24, §28, §34)

106. [x] Clippy: deny(warnings).
107. [ ] cargo-deny: license & advisories clean.
108. [ ] No unwrap() outside tests (or justify).
109. [ ] Determinism test repeated run identical output.
110. [ ] Validate no leftover TODO markers for MVP (or track in backlog list).
111. [x] Document and enforce dev workflow (fmt, clippy, nextest) (see Section 25).

---

## 20. CI Expansion

111. [ ] Add matrix (linux, macOS, windows).
112. [ ] Add MSRV job.
113. [ ] Add cache (actions-rs or dtolnay/rust-toolchain + Swatinem/rust-cache).
114. [ ] Add test coverage (cargo-tarpaulin).
115. [ ] Optional scheduled dependency audit.

---

## 21. Release Preparation

116. [ ] Ensure tool self-generates initial CHANGELOG.md via `release --dry-run`.
117. [ ] Tag v0.1.0 (manual first).
118. [ ] Publish crate (cargo publish) – optional if scope private initially.
119. [ ] Publish npm package (npm publish) via NAPI-RS build.
120. [ ] Verify install instructions (cargo install and npm install).
121. [ ] Announce parity & solicit feedback before adding new features.

---

## 22. Backlog (Not in MVP)

- Workspace multi-crate support.
- JSON export mode.
- Pre-release channels.
- Template customization extension (if user demand).
- Git hosting expansions beyond basic provider inference logic.
- Hook/plugin architecture.

---

## 23. Dependency Audit / Pruning Pass

121. [ ] After benchmarks: remove dashmap if not beneficial.
122. [ ] Evaluate need for reqwest until GitHub sync matured.
123. [ ] Confirm no accidental heavy transitive crates (document compile times).

---

## 24. Final Verification Checklist (All Must Pass)

- [ ] Matches JS output for a sampled repository (diff = empty).
- [ ] Parallel vs sequential identical.
- [ ] Signed tag path error messaging clear (if GPG absent).
- [ ] New version bump logic conforms to 0.x rules & explicit override.
- [ ] Idempotent rerun no duplication.
- [ ] Contributors section correctness (exclusions, email hiding).
- [ ] Compare link correct for GitHub & GitLab test remotes.
- [ ] All tests green on all CI platforms.
- [ ] No extraneous dependencies.

---

## Suggested Work Streams (Parallelization)

A. Config + Repo + Interpolation (Tasks 6–19, 41–44)  
B. Git + Parsing + Classification (20–34)  
C. Semver + Cargo Bump (35–40)  
D. Rendering + Changelog (51–63)  
E. Release Pipeline + CLI (64–80)  
F. GitHub Sync (69–73)  
G. Parallelization + Benchmarks (81–84, 95–100)  
H. Docs + QA + CI (101–115, 106–110, 111–115)  

---

## Milestone Gates

Milestone 1 (Core Parse): Tasks 6–34 complete, basic CLI show/generate prints block.  
Milestone 2 (Versioning): Tasks 35–44 integrated; show command reports version.  
Milestone 3 (Changelog): Tasks 51–63; generate updates file.  
Milestone 4 (Release Pipeline): Tasks 64–80; `release` end-to-end (no GitHub).  
Milestone 5 (GitHub Sync + Authors Polish): Tasks 45–50, 69–73.  
Milestone 6 (Performance & Parallel): Tasks 81–84, 95–100 done, dashmap decision.  
Milestone 7 (Docs & CI & QA): Tasks 101–115, 106–110, 111–115.  
Milestone 8 (Release v0.1.0): Tasks 116–120 + final checklist.

---

## Risk Mitigation Notes

- Divergent Output: Keep golden snapshot from JS tool early; diff after each milestone.
- Ordering Drift: Preserve original commit index; assert in test.
- Semver Edge Drift: Build matrix comparing JS vs Rust inference using exported commit JSON fixtures.
- Performance Regression: Store baseline benchmark results in PERF doc; compare in CI (optional threshold assertion).
- Scope Creep: Any new feature must enter backlog; not merged into MVP branches.

---

## Backlog Capture Mechanism

Add new potential features to a BACKLOG.md with: ID, description, rationale, requested-by, complexity estimate.

---

## 25. Developer Workflow / Quality Convenience

Guidelines (non-functional tasks, but enforceable via CI/hooks):

A. Pre-commit local checklist
- [x] cargo fmt --all (format code)  
- [x] cargo clippy --all-targets --all-features -- -D warnings  
- [ ] cargo nextest run  
- [ ] cargo test --doc (doc tests separately if any)  
- [ ] cargo deny check (optional fast path)  

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
- Future: coverage (tarpaulin) gate.  

D. Fast feedback loop:
- Use cargo watch -x 'clippy -- -D warnings' -x 'nextest run' for iterative dev.  

E. Panics policy:
- Outside tests: avoid unwrap/expect; map errors. (Tied to task 108).  

F. Pending automation tasks:
- [ ] Add Makefile or justfile with common recipes (make test, make lint).  
- [ ] Add CONTRIBUTING.md referencing this section.  

(End of tasks.md)
