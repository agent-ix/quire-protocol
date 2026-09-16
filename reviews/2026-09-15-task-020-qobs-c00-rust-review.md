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
typed unsupported capability surface and IT-002 SC-08 regression evidence
against repository idioms and the Rust-review checklist. The one independent
review finding was fixed before this artifact was finalized.

## Verdict

**PASS for Task-020.** No unresolved correctness, idiom, owner-boundary, panic,
unsafe, conversion, resource, determinism, trace or test-quality finding
remains in the reviewed subset.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Resolved: the initial change required a new producer contract to be repeated across the enum, `ALL` and several matches, so a future variant could compile while remaining absent from resolution. One declarative inventory now generates the enum, `ALL`, owner, contract, revision and schema selection; IT-002 keeps an independent literal table for the five C00 contract/digest selections. | `src/producer_contracts.rs::define_producer_contracts`; IT-002-SC-08 |
| FND-002 | medium | Resolved during mainline reconciliation: the obsolete stacked base carried a monolithic three-test IT-002 harness and assessment-authorization state that PR #89 intentionally removed when it narrowed Task-010 to SC-01..SC-04. The rebased change adds one focused SC-08 integration test, retains immutable resolution evidence only, and introduces no dead capability field or stale assessment claim. | `tests/it002_producer_contracts.rs`; `src/producer_contracts.rs::ProducerResolution`; Task-010; Task-020 |

## Rust Review

| Check | Result |
| --- | --- |
| Owner and API boundaries | All contract labels and schema bytes come from QObs exports. Protocol retains only its accepted revision/digests and a closed consumer capability status; it adds no QObs payload, decoder, repair or query semantics. |
| Inventory integrity | One macro invocation is the sole declaration-order and metadata source for all nineteen producer contracts. The resolver continues to derive its exact manifest from `ProducerContract::ALL`; the five new contract/digest pairs are independently asserted in IT-002. |
| Errors and safety | Repair and aggregate query are separate typed unsupported values and mint no manifest entry. No caller-controlled panic, `unsafe`, unchecked conversion, lock, async bridge or background work was added. Existing `expect` sites remain limited to nonempty static keys and established resolver invariants. |
| Bounds and allocation | The change adds only a fixed nineteen-element inventory and a fixed two-element unsupported inventory. Test-only collection is bounded by those arrays; no untrusted loop, recursion or retained payload was introduced. |
| Determinism | Declaration order remains stable, manifest entries retain their existing deterministic resolver order, and schema hashes are computed over the exact owner-exported byte arrays. |
| Test quality and trace | IT-002 SC-08 executes the public resolution path, exact/negative manifest controls, fifteen owner-schema digests, an independent five-entry C00 literal oracle and both unsupported variants. Assertions do not derive the new expected labels or digests from the production table. |

## Verification

The focused `it002_producer_contracts` target passes its SC-08 test and focused
Clippy passes with warnings denied. On the final reconciled candidate, the
repository-defined serialized gate passes its exact three commands: formatting;
locked all-target Clippy with warnings denied; and the full locked default test
suite (279 unit/integration tests plus 42 doc-tests, zero failed or ignored).

## Boundary

This review covers Task-020's compatibility successor only. It does not claim
QObs repair or aggregate-query adapter support, IT-002 SC-05 through SC-07,
Plan-001 or epic #14 complete.
