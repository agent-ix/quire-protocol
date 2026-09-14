---
id: SR-085
title: "Rust review of Task-001 strict-admission closeout"
type: SpecReview
analysis: code-review
scope: "quire-protocol issue #1; TC-001 traces and default-running admission tests on task/1-strict-admission-close"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Task-001, type: reviews }
---
# Rust Review of Task-001 Strict-Admission Closeout

## Summary

The remediated closeout is fit for promotion. It adds no production parser,
schema, profile, refusal vocabulary or version detector. The executable change
uses the existing public admission paths and the compiler owner's pinned
published handoff to close the missing baseline-selection and numeric controls.

## Verdict

**PASS** — no unresolved Rust idiom, test-oracle, panic, unsafe, resource,
dependency-identity, ownership-boundary or gate finding remains.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Resolved: the first raw-byte mutation helper searched every byte after a unique expression anchor, so a repeated value in a later expression could satisfy the wrong oracle. The helper now finds the enclosing expression boundary and requires exactly one target inside that record. | `tests/it001_v2_handoff.rs::mutate_after` |
| FND-002 | low | Resolved: the numeric control admitted and re-encoded the whole owner package but did not independently enumerate zero, ordinary normalized rational boundaries or a negative denominator. The positive inventory and adverse table now name those cases explicitly. | FR-001-AC-4; `numeric_edges_round_trip_and_invalid_forms_refuse_without_repair` |
| FND-003 | low | Resolved: the result-domain digest substitution already traced to FR-001-AC-4 but not to its owning TC-001 row. The existing discriminating production-path test now carries the bare TC-001 binding. | TC-001; `tc_006_result_digest_in_a_producer_slot_refuses` |

## Review Record

- All new `expect`, `unwrap_err`, assertions and panics are confined to tests.
  No production path, public API, allocation, lock, I/O, unsafe block, cast or
  error conversion changed.
- The caller's expected baseline is constructed from the independent selection
  sidecar. Identity, revision and digest are varied separately while the offer
  remains unchanged, and the unchanged control admits before and after them.
- Twelve malformed or out-of-domain numeric mutations are resealed only so the
  intended inner refusal can be reached. Each requires the production reader's
  exact stable code; none accepts, repairs, truncates or approximates a value.
- The owner corpus remains the sole authority for its 28 mutation cases and 21
  axes. The consumer freezes only counts/axis names needed to detect corpus
  shrinkage and uses A's public record types and constants for all reads.
- All 13 resource dimensions and both explicit version-entry paths continue to
  execute. No environment, network, clock, hosted workflow or copied corpus is
  involved.
- `Cargo.toml` remains `publish = false`; `DEPENDENCIES.md`, the manifest and
  lockfile agree on the exact revisions and license compatibility.

## Gate Evidence

- `cargo fmt --all -- --check`: pass.
- `cargo clippy --locked --all-targets --target-dir
  target-codex-backends -- -D warnings`: pass.
- `cargo test --locked --target-dir target-codex-backends`: 221 passed, 0
  failed, 23 explicitly ignored legacy version-1 fixture tests; 37 doc-tests
  passed.
- Targeted version-2 handoff: 17 passed, 0 failed, 0 ignored.
- Targeted digest-domain suite: 12 passed, 0 failed, 0 ignored.
- Toolchain: `rustc 1.98.1`, `cargo 1.98.1`.

All Cargo commands used `CARGO_BUILD_JOBS=1`, the shared heavy-check lock, and
the worktree-local `target-codex-backends` directory. No hosted CI ran.
