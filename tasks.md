# changelogen-rs Task Breakdown (Parity Port of @unjs/changelogen)

Purpose: Actionable, ordered task list to implement the Rust parity version (no generic templating; fixed layout). Mirrors JS behavior while adapting to Cargo.  
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
3. [x] Add base dependencies (no unused): git2, semver, clap, anyhow, thiserror, serde, serde_json, toml_edit, tracing, tracing-subscriber, rayon, jiff, reqwest (http requests), demand (prompting, kind of like huh? in go), dashmap (use if need hashmaps), git-conventional.
4. [x] Dev deps: insta, assert_fs, tempfile, proptest, criterion, cargo-deny, nextest.
5. [x] Set up CI workflow skeleton (Linux only first, then all OS).

---

## 1. Configuration Layer (§5, §6, §15)

6. [x] Implement defaults map exactly matching JS config.ts (types + emojis + semver).
7. [~] Support config file `changelogen.toml`.
8. [~] Support fallback `[package.metadata.changelogen]` in Cargo.toml.
9. [x] Implement environment token resolution precedence (CHANGELOGEN_TOKENS_GITHUB > GITHUB_TOKEN > GH_TOKEN).
10. [x] Implement overlay merge order: defaults < file(s) < CLI overrides.
11. [x] Support disabling a type via boolean false (TOML loader logic).
12. [x] Validate `newVersion` (semver parse).
13. [~] Warn on unknown keys (collect & log).
14. [x] Expose `ResolvedConfig` with normalized paths, resolved repo data (placeholder until repo module ready).

🧪 Tests:
- [ ] Default config equality snapshot.
- [ ] Override precedence (CLI wins).
- [ ] Boolean disabling of type.
- [ ] Token env override test.

---

## 2. Repo & Provider Resolution (§3.5, §5.1.4, §9)

15. [ ] Implement remote URL parsing (GitHub/GitLab/Bitbucket patterns).
16. [ ] Infer provider + domain from origin; fallback to config override.
17. [ ] Implement reference URL mapping spec (commit/issue/pull).
18. [ ] Implement compare link function (provider differences).
19. [ ] Add formatting helpers: `format_reference`, `format_compare_link`.

🧪 Tests:
- [ ] Parse SSH style URL.
- [ ] Parse HTTPS URL with .git suffix.
- [ ] Compare link generation per provider.

---

## 3. Git Layer (§7)

20. [ ] Implement repo detection + error if not a git repo.
21. [ ] Implement last tag discovery (semver tags; accept both vX.Y.Z and X.Y.Z; prefer latest by commit date).
22. [ ] Implement current ref (tag or branch HEAD).
23. [ ] Commit enumeration between (from, to].
24. [ ] Provide RawCommit struct (id, short_id, summary, body, author, timestamp).
25. [ ] Dirty working tree detection.
26. [ ] Utility: add & commit, create annotated/signed tag (shell fallback for signed).

🧪 Tests:
- [ ] No tags scenario.
- [ ] Single tag detection.
- [ ] Dirty tree detection.

---

## 4. Parsing & Classification (§7.4–7.7)

27. [ ] Implement conventional header parsing: type, optional scope, optional !, description (case-insensitive type).
28. [ ] Manual fallback if git-conventional crate unavailable.
29. [ ] Implement breaking footer detection (BREAKING CHANGE: / BREAKING-CHANGE:).
30. [ ] Implement issue / PR reference scanning (#\d+).
31. [ ] Implement Co-authored-by detection accumulating authors.
32. [ ] Scope mapping via config.scopeMap.
33. [ ] Normalize type to lowercase (§ commands/default.ts behavior).
34. [ ] Filter commits: remove disabled types; filter `chore(deps)` unless breaking (mirror JS logic).

🧪 Tests:
- [ ] Header with scope + bang.
- [ ] Footer-only breaking.
- [ ] Multiple issue references.
- [ ] Co-authored-by accumulation.
- [ ] chore(deps) filtered when not breaking.

---

## 5. Semver Inference & Version Bump (§4, §8)

35. [ ] Determine major/minor/patch flags using type semver + breaking flag.
36. [ ] Pre-1.0 adjustment: major→minor, minor→patch.
37. [ ] Default to "patch" if zero bump-worthy changes (match JS `|| "patch"`).
38. [ ] Apply explicit newVersion override if provided after inference.
39. [ ] Implement suffix logic (optional; confirm parity need).
40. [ ] Implement Cargo.toml version bump via toml_edit (preserve formatting).

🧪 Tests:
- [ ] Major inference via breaking.
- [ ] Minor inference via feat only.
- [ ] Patch inference via fix only.
- [ ] 0.x adjustments.
- [ ] Explicit newVersion override.
- [ ] Idempotent (same version returns false result).

---

## 6. Interpolation (§6, §10)

41. [ ] Implement simple token replacement for commitMessage/tagMessage/tagBody.
42. [ ] Support tokens: {{newVersion}}, {{previousVersion}}, {{date}}.
43. [ ] Unknown tokens remain as-is.
44. [ ] Date format: YYYY-MM-DD (UTC).

🧪 Tests:
- [ ] All tokens replaced.
- [ ] Unknown token retention.
- [ ] Date stable formatting.

---

## 7. Authors Aggregation (§8, §13)

45. [ ] Aggregate primary + co-authors.
46. [ ] Deduplicate (name+email).
47. [ ] Exclude authors (exact match list).
48. [ ] hideAuthorEmail support.
49. [ ] noAuthors flag suppression.
50. [ ] Preserve first-seen order.

🧪 Tests:
- [ ] Exclusion logic.
- [ ] hideAuthorEmail formatting.
- [ ] Dedup.

---

## 8. Rendering (Markdown) (§2, §9)

51. [ ] Section ordering = order of active types in config.
52. [ ] Include only non-empty sections.
53. [ ] Commit line formatting with or without scope.
54. [ ] Append references (linked if provider resolved).
55. [ ] Add compare link (if previous tag exists).
56. [ ] Contributors section conditionally.
57. [ ] Consistent trailing newline.
58. [ ] Deterministic ordering: sort commits chronologically & preserve insertion index tie-break.
59. [ ] Provide function `render_release_block`.

🧪 Tests:
- [ ] Snapshot standard release block.
- [ ] Empty sections trimmed.
- [ ] Compare link presence.
- [ ] Contributors ordering.

---

## 9. Changelog File Handling (§15, §19, §32)

60. [ ] Read existing file if present; else bootstrap "# Changelog".
61. [ ] Locate first release header; prepend new block above it.
62. [ ] Idempotence check: if same version already at top and identical body, skip write.
63. [ ] Provide `write_or_update_changelog` with diff summary.

🧪 Tests:
- [ ] Prepend first release.
- [ ] Subsequent release insertion.
- [ ] Idempotent rerun no duplication.

---

## 10. Release Pipeline (§11, §27)

64. [ ] Orchestrate steps: config load → git range → parse → classify → bump → render → write → commit → tag → GitHub sync (optional).
65. [ ] Dry run support (skip writes, commit, tag, network).
66. [ ] Exit code mapping (0 success, 3 no-change, etc.).
67. [ ] Summary output (version, counts, tag status).
68. [ ] Respect clean flag (abort if dirty when requested).

🧪 Tests:
- [ ] Dry run leaves files unchanged.
- [ ] Exit code 3 scenario (if implemented—validate parity).
- [ ] Signed tag attempt (simulate no key error path).

---

## 11. GitHub Release Sync (§12, §23)

69. [ ] Implement GET release by tag; create or update.
70. [ ] Fallback manual URL when token absent or error.
71. [ ] Redact token in logs.
72. [ ] Provide status struct to caller.
73. [ ] Optionally `--github` subcommand to resync existing tag with current body.

🧪 Tests (network mock or feature-gated):
- [ ] Manual fallback without token.
- [ ] Update path after create.

---

## 12. CLI Design (§14)

74. [ ] Implement `show` (print inferred next version).
75. [ ] Implement `generate` (print block; optional --write).
76. [ ] Implement `release` (full pipeline).
77. [ ] Implement `github` (sync only) if maintained.
78. [ ] Global flags: --from, --to, --new-version, --sign, --output, --no-authors, --exclude-author, --cwd, --dry-run, --yes, --clean.
79. [ ] Verbosity flags or RUST_LOG integration.
80. [ ] Helpful `--help` docs per subcommand.

🧪 Tests:
- [ ] CLI argument parsing snapshot (clap test harness).
- [ ] Unknown subcommand error.

---

## 13. Parallel Parsing (§17)

81. [ ] Implement threshold env override & CLI override (optional).
82. [ ] Use rayon for parse/classify only when commit_count >= threshold.
83. [ ] Maintain original index for stable ordering.
84. [ ] Provide debug logs indicating mode.

🧪 Tests:
- [ ] Output identical sequential vs parallel (snapshot diff).
- [ ] Force parallel with small set (env var) still identical.

---

## 14. Logging & Telemetry (§18)

85. [ ] Integrate tracing subscriber (env filter).
86. [ ] Add spans: collect_commits, parse_classify, infer_version, render, write, tag, github_sync.
87. [ ] Debug-level per-commit classification log.
88. [ ] Provide minimal JSON log format stub (optional backlog).

🧪 Tests:
- [ ] Smoke log initialization (no panic).
- [ ] Verbose toggle effect.

---

## 15. Error Handling (§18, §21)

89. [ ] Define Error enum (Config, Git, Network, IO, Semantic).
90. [ ] Map to exit codes.
91. [ ] Wrap CLI main with error -> stderr formatted line.
92. [ ] Avoid panics outside unrecoverable invariants.

🧪 Tests:
- [ ] Config parse failure case.
- [ ] No git repo detection.

---

## 16. Authors & Contributors Edge Enhancements

93. [ ] Normalize unicode (NFC) for consistent grouping.
94. [ ] Provide optional aliasing (future/backlog marker).

---

## 17. Benchmarks (§22, §33)

95. [ ] Benchmark synthetic commit generation utility.
96. [ ] Implement parse_seq_vs_parallel benchmark.
97. [ ] Implement render_block benchmark (vary commit counts).
98. [ ] Implement version_inference benchmark.
99. [ ] Document baseline results.
100. [ ] Evaluate dashmap effect; remove if <5% improvement (then: update Cargo.toml).

---

## 18. Documentation (§22, §29)

101. [ ] README: parity statement, quick start, differences vs JS (Cargo vs npm).
102. [ ] CONTRIBUTING: dev setup, MSRV, tests, benchmarks, release instructions.
103. [ ] PERF docs: initial benchmark table.
104. [ ] PARITY_SPEC inclusion & cross-link tasks.md.
105. [ ] Changelog bootstrapped by tool itself for first release.

---

## 19. Quality Gates (§24, §28, §34)

106. [ ] Clippy: deny(warnings).
107. [ ] cargo-deny: license & advisories clean.
108. [ ] No unwrap() outside tests (or justify).
109. [ ] Determinism test repeated run identical output.
110. [ ] Validate no leftover TODO markers for MVP (or track in backlog list).

---

## 20. CI Expansion

111. [ ] Add matrix (linux, macOS, windows).
112. [ ] Add MSRV job.
113. [ ] Add cache (actions-rs or dtolnay/rust-toolchain + Swatinem/rust-cache).
114. [ ] Add test coverage (grcov or cargo-llvm-cov optional).
115. [ ] Optional scheduled dependency audit.

---

## 21. Release Preparation

116. [ ] Ensure tool self-generates initial CHANGELOG.md via `release --dry-run`.
117. [ ] Tag v0.1.0 (manual first).
118. [ ] Publish crate (cargo publish) – optional if scope private initially.
119. [ ] Verify install instructions (cargo install).
120. [ ] Announce parity & solicit feedback before adding new features.

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

(End of tasks.md)
