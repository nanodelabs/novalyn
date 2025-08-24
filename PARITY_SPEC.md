# Parity Spec: changelogen-rs vs @unjs/changelogen

Goal: Achieve output parity (format, ordering, inference) with the JS version while adapting to Rust tooling (Cargo, git2) and keeping a fixed template (no user templating in MVP).

Scope (MVP):
- Config surface identical where meaningful (types, emojis, semver directives, token precedence)
- Commit classification logic mirrors JS (conventional commits + breaking detection + filtering rules)
- Version inference rules including 0.x adjustments
- Markdown release block layout + compare links + contributors list
- Changelog insertion & idempotence
- Release pipeline orchestration excluding advanced GitHub interactions (basic sync only)

Non-Goals (Backlog):
- Workspace / multi-crate aggregation
- Arbitrary template customization
- Extended hosting providers beyond GitHub/GitLab/Bitbucket baseline

Verification Strategy:
1. Capture sample repository commits & JS tool output as golden snapshots.
2. Diff Rust output after each milestone gate.
3. Property-based tests around semver inference & parsing edge cases.
4. Benchmarks to ensure no >10% regression after optimizations.

Notes:
- Any deviation must be documented in README differences section.
- Keep deterministic ordering tests to prevent drift.

(Initial stub – expand with concrete mapping tables as implementation proceeds.)
