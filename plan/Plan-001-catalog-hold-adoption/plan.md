---
id: Plan-001
title: "QSL-1802 — checked-package catalog hold adoption"
type: Plan
status: not_started
relationships:
  - target: ix://agent-ix/quire-protocol/FR-043
    type: references
  - target: ix://agent-ix/quire-protocol/NFR-004
    type: references
---
# Implementation Plan: checked-package catalog hold adoption

## Requirements Summary

- [ ] FR-043: retain existing AC-1 through AC-8 behavior; implement AC-9 unchanged owner-error propagation.
- [ ] NFR-004: preserve the ordered four-command verification lane, with the explicit user thermal override below.

## Readiness and Boundaries

Reviewed spec PR #139 is merged. Independent reviews SR-2127004 through SR-2127007 reside in reviews/qsl-1802-{code-review,base,ears-conformance,dependency}.md. Atomicity, ownership, dependencies, risks, failure domains and verification are resolved there. Protocol owns its adapter and lock; Contract IR owns FR-038, CatalogReadError and catalog reading. Risk is bounded API/lock adoption: an unhandled result must not panic or be mistaken for invalid offered bytes. The imported opaque error remains unchanged; no consumer constructor, catalog copy or new taxonomy is required. Existing production catalog validation belongs to the owner. No new API is justified merely to force this failure in a Protocol test.

## Dependency Graph

- Owner Contract IR FR-038 and reviewed Protocol FR-043 precede the adapter change: dispatch must be understood before matching its result.
- Task frontmatter defines the serial implementation-to-review dependency. Lock and adapter changes have one writer; there is no useful parallel source track.
- NFR-004 constrains promotion of the unchanged implementation state.

## Test Plan

- TC-149: run the existing checked-package admission tests against refreshed dependencies, preserving FR-043-AC-1 through AC-8; inspect the exhaustive production arm for AC-9 unchanged payload and early return before selection, identity or equality checks. Do not claim runtime catalog corruption coverage or invent an opaque-owner-error fixture. Existing tagged tests retain #[trace("TC", "FR-AC")] attributes; any real added test uses actual criterion IDs.
- TC-012: independent review and ordered four-command verification in the declared feature lane. Record application version and actual command outcomes; distinguish unrelated baseline validation errors from scope regressions.

## Remaining Work

### Track A: Critical Path (serial)

- A1 = Task-001 owner-error adoption and lock refresh — small; exit: current owner API compiles and catalog unavailability returns unchanged before later checks.
- Gate = Task-002 independent review and local verification — small; pass: zero unresolved findings and all four commands pass on one unchanged head; failures are resolved in the same PR before promotion.

## Parallel Execution Summary

Task-001 → Task-002. No independent source track: both stages concern the same adapter and lock state.

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
| --- | --- | --- | --- | --- |
| Task-001 | A | FR-043 | TC-149 | not_started |
| Task-002 | Gate | FR-043, NFR-004 | TC-149, TC-012 | not_started |

## Coordination Rules

Implementation begins only after explicit human plan acceptance. Use a fresh worktree and one small PR referencing QSL-1802. Refresh the existing moving-main dependency lock to current admitted owner heads; do not add revision pins, tracking-hash catalogs, compatibility layers or copied repository files. Do not touch contract-ir #1010, RT #218 or terra's work. Preserve source/spec ownership and independently review the resulting change before a head-matched merge. The user-required exact GitHub head guard and merge-request identity are transient operator checks; this bundle introduces no authored revision ledger or recorded SHA catalog.

For later Cargo work, the user's explicit thermal rule overrides NFR-004's jobs=1, /tmp/quire-heavy-check.lock and target-codex-backends procedure: every Cargo command uses flock /tmp/titan-cargo.lock nice -n10 ionice -c3 env RUSTUP_TOOLCHAIN=1.98.1 CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RUSTC_WRAPPER= CARGO_TARGET_DIR=<worktree>/target. Do not claim literal NFR-004-AC-3/9 procedure compliance. No make ci, Kani or timeout wrappers. Preserve command order: fmt --all -- --check; clippy --locked --all-targets -- -D warnings; test --locked; doc --locked --no-deps with RUSTDOCFLAGS=-D warnings. All four commands run serially on one unchanged state using the own-worktree target; delete target when finished. Tests that invoke cargo metadata require the user-documented pgrep-clear/no-flock exception to avoid deadlock, with that exception recorded as such.
