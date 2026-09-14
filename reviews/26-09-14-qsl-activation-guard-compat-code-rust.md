---
id: SR-087
title: "Code and Rust review of the QSL activation-guard compatibility repin"
type: SpecReview
analysis: code-review
scope: "quire-protocol#59; Cargo.toml; Cargo.lock; DEPENDENCIES.md; tests/it001_dependency_inventory.rs; tests/it001_v2_handoff.rs"
review_set: subset
relationships:
  - target: ix://agent-ix/quire-protocol/IT-001
    type: reviews
  - target: ix://agent-ix/quire-protocol/NFR-004
    type: references
---

## Summary

`/code-review` and `/rust-review` examined the complete QSL `f1700a9`
compatibility repin. The resolved graph contains one QSL crate identity, the
existing producer handoff and strict reader behavior are unchanged, and the
exact revision is asserted by executing Rust rather than documentation alone.

## Verdict

**PASS** — no actionable code, Rust, dependency, licensing, test-alignment, or
scope finding remains.

## Rust review

- No production behavior, type, schema, wire string, parser, evaluator, unsafe
  surface, panic path, conversion, lock, async path or allocation changed.
- `Cargo.toml` and `Cargo.lock` select the same immutable QSL merge; `cargo tree`
  resolves one QSL package identity throughout the graph.
- The existing `it_001_resolves_one_frontend_in_the_whole_graph` and exact
  `FRONTEND_REVISION` assertion prevent a second frontend or stale transitive
  owner from entering silently.
- QSL retains its `AGPL-3.0-only` grant, which remains compatible with this
  crate's `AGPL-3.0-or-later` decision. No artifact is copied or vendored.
- Test-only changes update exact provenance comments and constants; they add no
  test-local owner vocabulary or tautological behavior.

## Gates

| Gate | Result |
| --- | --- |
| Mandated worktree-local format gate | pass |
| Mandated all-target clippy gate with warnings denied | pass |
| Mandated default test gate | pass: 221 tests plus 37 doctests; 23 declared external-fixture tests ignored |
| Pinned v2 handoff suite | 17/17 pass, including all published mutations and resource boundaries |
| Dependency identity inventory | 3/3 pass; one QSL `f1700a9` identity |

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No unresolved code or Rust finding remains. | #59; IT-001; NFR-004 |
