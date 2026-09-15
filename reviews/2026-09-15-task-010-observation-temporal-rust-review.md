---
id: SR-118
title: "Rust review of the Task-010 observation and temporal integration"
type: SpecReview
analysis: code-review
scope: "Task-010 / quire-protocol#12 over the stacked candidate and its review fixes"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-010, type: reviews }
  - { target: ix://agent-ix/quire-protocol/IT-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-009, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: reviews }
---
# SR-118: Rust Review of the Task-010 Observation and Temporal Integration

## Summary

Reviewed the closed QObs/QSL producer-contract inventory, request-bound
resolution, activation-derived integration, shared resource accounting,
batch/incremental correspondence and IT-002 harness against the repository's
Rust and owner-boundary rules. Every in-scope finding was fixed before this
artifact was finalized.

## Verdict

**PASS for Task-010's producer-integration allocation.** No unresolved Rust,
owner-vocabulary, panic, unsafe, conversion, resource, identity, trace or test-
quality finding remains in the reviewed subset. Final FR-017 record composition
and integration-level refusal-axis validation remain Task-018-owned.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Resolved: the graph-derived obligation identity omitted workflow and repeat ordinals, so two legal occurrences of one event control collapsed into `DuplicateActivatedObligation`. The identity now includes byte-exact workflow identity plus fixed-width repeat ordinals, and IT-002 exercises two occurrences through early and final prefixes. | `src/conformance/kernel.rs::derived_obligation_id`; IT-002-SC-05b |
| FND-002 | high | Resolved: a completed bundle initially retained assessment usage but dropped the exact producer resolution and caller grants, so Task-018 could not attribute the outcome without caller reconstruction. The public composed path now retains a typed resolution snapshot, selected contracts, resolver usage, caller budget and operation usage while leaving final FR-017 composition private to Task-018. | `src/producer_contracts.rs::ProducerResolutionEvidence`; `src/assessment.rs::AssessmentBundle`; IT-002-SC-07a |
| FND-003 | medium | Resolved: IT-002 accepted any nonempty global-premise defect collection, allowing missing, duplicate, unexpected and cross-wired cases to collapse. Each mutation now asserts its exact closed `GlobalPremiseDefectKind`, and `NotRequired` asserts the exact conformance refusal. | `tests/it002_observation_temporal.rs::it_002_sc_01_through_04_flow_owner_facts_to_one_canonical_result`; IT-002-SC-04 |
| FND-004 | medium | Resolved: canonical-result assertions sampled decision progress and surrounding closure only. The integration now independently checks all four scoped progress/closure identities plus completeness, global state and exact premises. | `tests/it002_observation_temporal.rs::it_002_sc_01_through_04_flow_owner_facts_to_one_canonical_result`; IT-002-SC-03 |

## Rust Review

| Check | Result |
| --- | --- |
| Owner boundaries | All observation, subject, relationship, temporal request/result, progress, closure, completeness, ledger, clock, capture and availability values come from QObs or QSL types and constants. Protocol introduces only its own closed contract selection, resolution evidence and request identity; no producer schema, enum or decoder is mirrored. |
| Public capability surface | `ProducerResolution` retains Task-013's constructor-private capabilities and exposes only immutable evidence. Assessment authorization validates every exact owner/contract/revision against the same componentwise request identity; foreign Engine reuse refuses. Debug output omits the private capability. |
| Errors and panics | Recoverable manifest, capability, conformance and resource states return typed `OperationOutcome`/refusal variants. Production `expect` sites are limited to compile-time nonempty constants, a validated nonempty request wrapper and immediately established resolver invariants; caller-controlled malformed values do not reach them. No `unwrap`, unsafe, recursion, blocking bridge, lock or background task was added. |
| Numeric and identity boundaries | Request components and counts use fixed-width big-endian length framing; slot/control conversions are checked at their owner domains. The only `usize` to `u128` casts are widening internal identity encodings. Workflow and repeat ordinals now disambiguate otherwise equal event-control occurrences. |
| Bounds | Resolution charges exact required-entry count and its distinct deadline; composed assessment retains one shared accountant across conformance, dispatch, result production and strict reading. Tests admit exact grants, refuse representable one-short grants on their exact dimension and complete under a fresh budget. |
| Determinism | Producer contracts have one closed exhaustive inventory; resolved entries remain lexicographically ordered by Task-013, selected contracts retain declaration order, runtime bindings sort by admitted slot, and permitted arrival permutations compare semantic reports rather than allocation or iteration order. |
| Test quality and trace | Three default integration tests execute real public owner and Protocol paths. Assertions discriminate exact revisions, contract populations, refusal codes, defect kinds, axes, provenance, early settlement, batch/incremental equality, resource dimensions and retained attribution. No ignored, clock-sleep, ambient-service or mock-everything test was added. |
| Repository constraints | New implementation and tests are AGPL-3.0-or-later Rust, `publish = false` remains unchanged, no feature/workflow/hosted-CI surface changed, and the user-owned untracked activation review was not modified. |

## Verification

The focused IT-002 lane passes 3 tests with zero failure or ignore. On the final
reconciled candidate, the repository-defined serialized gate passes its exact
three commands: formatting; locked all-target Clippy with warnings denied; and
the full locked default test suite, including 109 unit tests, every integration
target and 43 doc-tests.

## Boundary

This PASS covers Task-010's exact producer resolution and its ten non-record
IT-002 scenario units. It does not claim SC-07a's final evidence record,
SC-07b's integration-level record-set invariant, Task-018, Plan-001 or epic #14
complete.
