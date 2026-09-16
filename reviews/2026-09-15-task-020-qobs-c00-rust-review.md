---
id: SR-127
title: "Rust review of the Task-020 QObs C00 consumer successor"
type: SpecReview
analysis: code-review
scope: "Task-020 / quire-protocol#12 over the QObs C00 pin and producer-contract inventory"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-020, type: reviews }
  - { target: ix://agent-ix/quire-protocol/IT-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-009, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-006, type: reviews }
---
# SR-127: Rust Review of the Task-020 QObs C00 Consumer Successor

## Summary

Reviewed the exact dependency pin, owner-backed contract/schema inventory,
typed unsupported capability surface, enforced Task-010 assessment boundary
and IT-002 SC-08 regression evidence against repository idioms and the
Rust-review checklist. The independent high/medium code findings were fixed
before this artifact was finalized.

## Verdict

**PASS for the Task-020 code subset and promotion.** No unresolved
correctness, idiom, owner-boundary, panic, unsafe, conversion, resource,
determinism, trace or test-quality finding remains. The exact accepted QObs PR
#27 merge is pinned consistently, so the former external promotion hold is closed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Resolved: the initial change required a new producer contract to be repeated across the enum, `ALL` and several matches, so a future variant could compile while remaining absent from resolution. One declarative inventory now generates the enum, `ALL`, owner, contract, revision and schema selection; IT-002 keeps an independent literal table for all twenty-one selected contracts, including compiled `/2` and activation mapping `/3`. | `src/producer_contracts.rs::define_producer_contracts`; IT-002-SC-08 |
| FND-002 | high | Resolved: producer resolution was exercised only as a standalone SC-08 API while the real Task-010 producer reads and atomic assessment remained callable without it. Public `start_*_after_resolving` boundaries now mint an unforgeable one-use permit only after resolution and accept input bundles whose fields are private and whose constructors consume that permit. The fixed-size request identity binds package, static selection, ordered claims, activation mapping, qualified subject, temporal subject, request, result and (when used) checked predicate before a private Engine constructor runs. Raw construction and detached resolution-to-engine methods are not public, and `ResolvedAssessmentEngine` exposes explicit operations rather than dereferencing to Engine. | `src/assessment.rs`; `src/producer_contracts.rs`; TC-002; TC-003; TC-006 |
| FND-003 | medium | Resolved: SC-08's helper discarded affected identity and did not cover sidecar non-authority, sibling repin independence, unresolved staging, or real read ordering. SC-08 now checks the exact manifest mutations while TC-002 drives the actual activated boundary: an unresolved manifest invokes no QSL result reader, the strict result read occurs inside the permitted loader after resolution, and independent claim/mapping/subject/temporal request/result retargeting refuses with `capability.request-mismatch`; TC-006 separately proves checked-predicate retargeting refuses. | IT-002-SC-08; TC-002; TC-006 |
| FND-005 | high | Resolved: interface resolution previously accepted any nonempty owner assertion and retained the caller's entry clone after its deadline check. The adapter requirement now selects the exact static immutability assertion, rejects wrong kind, revision or assertion independently, and clones the accepted entry inside the deadline unit. | `src/version_lock.rs`; IT-002-SC-08 |
| FND-004 | low | Resolved external prerequisite: exact QObs pin `9240063` is accepted owner PR #27's merge commit. The constant, dependency record, plan and tests select the same revision. | `QOBS_REVISION`; `DEPENDENCIES.md`; Task-020; QObs PR #27 |

## Rust Review

| Check | Result |
| --- | --- |
| Owner and API boundaries | All contract labels and schema bytes come from owner exports. Protocol retains only exact accepted revisions/digests and a closed consumer capability status; it adds no producer payload, decoder, repair or query semantics. Atomic assessment resolves before minting the loader permit, then verifies every assessment selection before constructing its private Engine wrapper. |
| Inventory integrity | One macro invocation is the sole declaration-order and metadata source for all twenty-one producer contracts. The resolver derives its manifest from `ProducerContract::ALL`; an independent IT-002 literal table asserts every owner, contract and revision. |
| Errors and safety | Repair and aggregate query are separate typed unsupported values and mint no manifest entry. No caller-controlled panic, `unsafe`, unchecked conversion, lock, async bridge or background work was added. Existing `expect` sites remain limited to nonempty static keys and established resolver invariants. |
| Bounds and allocation | The change adds a fixed twenty-one-element inventory and fixed two-element unsupported inventory. The retained request identity is constructed only from fixed-size digests; exact interface revision/assertion selection and accepted-entry cloning execute inside resolution deadline units. |
| Determinism | Declaration order remains stable, manifest entries retain their existing deterministic resolver order, and schema hashes are computed over the exact owner-exported byte arrays. |
| Test quality and trace | IT-002 SC-08 executes exact/negative manifest controls, exact affected identities, non-authoritative sidecars, sibling independence, fifteen owner-schema digests, an independent twenty-one-entry literal oracle and both unsupported variants. TC-002 proves a real strict owner read occurs only inside the post-resolution loader and independently discriminates every activated request axis; TC-003 and TC-006 exercise producer-dependent operations only through the resolved wrapper, including checked-predicate non-retargetability. |

## Verification

The focused SC-08, Task-002 activated-boundary and Task-006 canonical-assessment
tests pass, and focused Clippy passes with warnings denied. On the final
reconciled candidate, the repository-defined serialized gate passes its exact
three commands: formatting; locked all-target Clippy with warnings denied; and
the full locked default test suite (279 unit/integration tests plus 42
doc-tests, zero failed or ignored). Native Quoin 0.23.1 validation reports zero
findings. Supplementary strict rustdoc remains blocked by the repository's
pre-existing broken/private intra-doc links outside this change; unconfigured
`cargo deny` likewise rejects the repository's existing license/source graph
under its default empty allowlist and is not a declared local gate.

## Boundary

This review covers Task-020's compatibility successor only. It does not claim
QObs repair or aggregate-query adapter support, IT-002 SC-05 through SC-07,
Plan-001 or epic #14 complete.
