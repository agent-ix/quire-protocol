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

**PASS for the Task-020 code subset; promotion blocked.** No unresolved
correctness, idiom, owner-boundary, panic, unsafe, conversion, resource,
determinism, trace or test-quality finding remains. QObs PR #27 is still draft,
so this code verdict does not authorize merging Protocol PR #93.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Resolved: the initial change required a new producer contract to be repeated across the enum, `ALL` and several matches, so a future variant could compile while remaining absent from resolution. One declarative inventory now generates the enum, `ALL`, owner, contract, revision and schema selection; IT-002 keeps an independent literal table for the five C00 contract/digest selections. | `src/producer_contracts.rs::define_producer_contracts`; IT-002-SC-08 |
| FND-002 | high | Resolved: producer resolution was exercised only as a standalone SC-08 API while the real Task-010 producer reads and atomic assessment remained callable without it. The producer-reading Engine constructors and `Engine::assess_activated_after_resolution` are now private; only an unforgeable `ProducerResolution` can start a public `ResolvedAssessmentEngine`, and only that wrapper exposes atomic assessment. `resolve_before_producer_read` reports prerequisite refusal as typed `Unexecutable`, with resource incompleteness separate. | `src/assessment.rs::ProducerResolution::start_result_assessment`; `src/assessment.rs::ResolvedAssessmentEngine`; `src/producer_contracts.rs::ProducerReadiness` |
| FND-003 | medium | Resolved: SC-08's helper discarded affected identity and did not cover sidecar non-authority, sibling repin independence, unresolved-to-unexecutable staging, or read ordering. It now checks independently framed exact key identities for every mutation, two independently repinned non-authoritative sidecars, exactly one changed producer entry, and zero callback reads for unresolved/repointed manifests before one read after exact resolution. | `tests/it002_producer_contracts.rs::it_002_sc_08_resolves_only_the_exact_scenario_contract_inventory` |
| FND-004 | low | External prerequisite, not a Rust defect: exact QObs pin `581d98f` is the head of open draft owner PR #27. The constant, dependency record and plan now call it a candidate, and Protocol PR #93 remains non-mergeable by project decision until owner acceptance. | `QOBS_REVISION`; `DEPENDENCIES.md`; Task-020; QObs PR #27 |

## Rust Review

| Check | Result |
| --- | --- |
| Owner and API boundaries | All contract labels and schema bytes come from QObs exports. Protocol retains only its exact candidate revision/digests and a closed consumer capability status; it adds no QObs payload, decoder, repair or query semantics. Atomic assessment is inaccessible until exact resolution returns its constructor-private wrapper. |
| Inventory integrity | One macro invocation is the sole declaration-order and metadata source for all nineteen producer contracts. The resolver continues to derive its exact manifest from `ProducerContract::ALL`; the five new contract/digest pairs are independently asserted in IT-002. |
| Errors and safety | Repair and aggregate query are separate typed unsupported values and mint no manifest entry. No caller-controlled panic, `unsafe`, unchecked conversion, lock, async bridge or background work was added. Existing `expect` sites remain limited to nonempty static keys and established resolver invariants. |
| Bounds and allocation | The change adds only a fixed nineteen-element inventory and a fixed two-element unsupported inventory. Test-only collection is bounded by those arrays; no untrusted loop, recursion or retained payload was introduced. |
| Determinism | Declaration order remains stable, manifest entries retain their existing deterministic resolver order, and schema hashes are computed over the exact owner-exported byte arrays. |
| Test quality and trace | IT-002 SC-08 executes the public staged path, exact/negative manifest controls, exact affected identities, non-authoritative sidecars, sibling independence, pre-read refusal, fifteen owner-schema digests, an independent five-entry C00 literal oracle and both unsupported variants. TC-006 exercises the actual atomic assessment only through the resolved wrapper. Assertions do not derive expected labels or digests from the production table. |

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
