---
id: SR-101
title: "Rust review of the Task-017 outcome and refusal foundation"
type: SpecReview
analysis: code-review
scope: "Task-017 / quire-protocol#62 candidate diff over 8a2d915"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-017, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-014, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-016, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-011, type: references }
---
# SR-101: Rust review of the Task-017 outcome and refusal foundation

## Summary

Reviewed the complete Task-017 Rust, test and tracking diff over architecture
baseline `8a2d915`: the public operation/claim outcome algebra, Protocol-owned
refusal vocabulary, local/source cause interiors, construction boundary,
existing envelope adapters and TC-011 regression suite. Every in-scope finding
was fixed before this artifact was finalized.

## Verdict

**PASS.** The final candidate has no unresolved Rust, test-quality, ownership,
panic-surface, resource, trace or gate-integrity finding.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No unresolved findings | - |

## Review-cycle remediations

| Finding | Severity | Resolution |
| --- | --- | --- |
| Refusal, axis and source-cause constructors were public, allowing a downstream caller to fabricate an envelope or label arbitrary bytes as a known source cause. | high | Made the construction path crate-private while keeping all consumer accessors public; known-source retention remains available only to exact internal owner adapters. |
| The shared result refusal constructor labeled strict result and mapping readers as `result-assembly`. | high | Split the internal constructor into assembly/read paths, threaded the actual stage through shared preflight and mapping helpers, and added assertions for assembly, result-read, conformance and claim-dispatch routes. |
| Replacing the original TC-011 file dropped mutation controls for catalog root/domain/grammar drift, quote/comment scanner handling, stale exemptions, inverted spans, identity components and hashing. | medium | Restored the full regression inventory and expanded it with the new outcome, stage/domain, typed-cause, source-contract and collision cases; the focused lane is now 30 tests. |
| Boxing the now-box-backed `ProtocolRefusal` again in subsystem refusals added a redundant allocation. | low | Store `ProtocolRefusal` directly in conformance, claim and result refusals; the envelope itself owns the single size-stabilizing box. |
| New tests carried `Task-017` through the runtime trace attribute even though the active Quire model mints requirement/test rows, producing 29 new untracked symbols. | medium | Retained Task-017 in human trace comments and removed it from machine trace attributes; current coverage reports zero Task-017 untracked symbols. |
| The frozen IT-001 source-module and TC-012 conditional-site inventories did not admit the new `outcome.rs` module and `src/refusal/mod.rs` test module, so the complete local test gate refused the candidate. | medium | Added both deliberate sites to their exact frozen inventories; the focused IT-001/TC-012 lanes and the repeated complete unchanged-head gate pass. |

## Rust review

| Check | Result |
| --- | --- |
| Ownership and public construction | FR-008 owns the seven domains, ten stages, local cause set and envelope. `ProtocolRefusal`, `RefusalAxis`, known/unknown source causes and retained sibling construction have no public constructor. No upstream type or wire schema is mirrored. |
| Closed matching | Domain, stage and local-cause matches are exhaustive with no wildcard; the stage-to-domain map is one total `const fn` and every one of its 70 valid/invalid offered pairs is exercised. |
| Outcome integrity | `OperationOutcome<T>` has exactly completed/refused/resource-incomplete variants. `ClaimOutcome<Row>` makes row versus claim-local refusal exclusive. Resource exhaustion is absent from `ProtocolCauseKind`. |
| Refusal identity | Equality/hash use the same typed field tuple and exclude rendering. A separate interior comparison excludes both axis and rendering so an axis key cannot make collision detection tautological. Mutation tests change every identity component. |
| Source retention | Known source causes require exact independently selected contract identity/revision before retention; unknown contract/code/stage/locus/payload bytes remain opaque in a distinct variant. No spelling similarity or nearest-version mapping exists. |
| Errors and panic surface | Fallible external-byte constructors return typed mismatch errors. New production `expect` calls are limited to crate-owned, audited code/stage invariants; no caller input reaches them. No `unwrap`, `panic!`, indexing, unchecked integer cast, recursion or unbounded input loop was added. |
| Future private foundation | Precise `cfg_attr(not(test), expect(dead_code, reason = ...))` attributes cover only constructor-private APIs whose consumers are assigned to Tasks 013 and 002–006. They are absent in test builds and become unfulfilled warnings when production adoption makes them obsolete. No `allow` was added. |
| Bounds and side effects | Outcome/refusal construction adds no allocation, retention, traversal, operation or output site beyond the already-inventoried retained bytes and vectors. No I/O, async, lock, process, network, persistence or clock path was added. |
| Test quality | Tests exercise real constructors and accessors, all closed members, every invalid stage/domain pair, exact source mismatch, cause order/multiplicity, zero-detail retention, pairwise outcome inequality, independent claims, rendering/hash invariance and killing catalog mutations. No ignored, clock-based, conditional-assertion or mock-only test was added. |
| Gate integrity | Cargo manifest, lockfile, dependencies, features, lints, workflows and `resources/native-v1/` are unchanged. `publish = false` remains in force and no hosted workflow was dispatched. |

## Verification

The final unchanged candidate passes the repository-defined local gate in its
required order with `CARGO_BUILD_JOBS=1`, `flock
/tmp/quire-heavy-check.lock`, `--locked`, and the worktree-local
`--target-dir target-codex-backends`: formatting check, all-target Clippy with
warnings denied, then the complete test suite. The 30-test TC-011 foundation
and catalog lane passes with zero ignored tests. Specification, plan and review
artifacts validate with Quire; repository-wide coverage is 128/341 rows, with
zero Task-017 untracked symbols.

## Boundary

Task-017 establishes the composed foundation; it does not pretend the existing
standalone `bounds::inventory::OperationOutcome<T, E>` kernel transports have
already been replaced. Tasks 013 and 002–006 consume the new private
constructors and Task-006 removes the residual per-module failure routes and
adds full public-path cause/resource coverage. That assigned work is not an
unresolved Task-017 finding.
