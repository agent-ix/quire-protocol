---
id: SR-130
title: "Gap analysis of Task-021 replay and resource completion"
type: SpecReview
analysis: gap-analysis
scope: "Task-021 / quire-protocol#12, IT-002 SC-05 and SC-06, TM-001"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-021, type: reviews }
  - { target: ix://agent-ix/quire-protocol/IT-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: references }
---
# SR-130: Gap Analysis of Task-021 Replay and Resource Completion

## Summary

Audited Task-021's plan state, IT-002 SC-05/SC-06 scenario text, requirement
and matrix trace, production surface, negative controls, bounds and accepted
QSL pin after Rust-review remediation. The targeted implementation allocation
is complete while Task-018 retains SC-07 and final scenario-evidence ownership.

## Verdict

**PASS for Task-021 promotion.** The five targeted IT-002 scenario units and
their implementation, adverse evidence and resource boundaries are complete.
This subset verdict is not a verdict on all of IT-002, Plan-001 or epic #14.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Resolved: activation-derived semantic frontends could bypass the evidence correlation enforced by the composed Engine. The shared kernel now validates every batch fact and incremental prefix/final request, and TC-002 uses owner-matched open, incomplete and failed QSL requests. | `src/conformance/kernel.rs`; TC-002; IT-002-SC-05 |
| FND-002 | high | Resolved: evidence hashing and activation-model derivation were absent from the production inventory and occurred before the request accountant. Five reconciled source sites and exact boundary tests now cover every added traversal, retained node, identity input, bounded comparison and stable allocation. | `src/bounds/inventory.rs`; IT-002-SC-06; NFR-002 |
| FND-003 | medium | Resolved: the original SC-05 test was too weak to establish the authored non-singleton equal-time, batch/incremental, early-settlement and divergence obligations. It now carries three distinct owner observations, both permutations, production semantic comparison, matched premise mutation and typed divergence. | IT-002-SC-05a..SC-05d; FR-004-AC-4; FR-026-AC-2; FR-041-AC-1..AC-4 |
| FND-004 | medium | Resolved: evidence-coordinate, individual-axis and repeat-ordinal mutations were missing. The targeted negative corpus now fails each evidence coordinate and each temporal axis independently, and changes to repeat identity alter the derived obligation identity. | `src/conformance/kernel.rs`; `tests/it002_observation_temporal.rs` |
| FND-005 | low | The wider repository remains intentionally incomplete outside this subset. IT-002 SC-07a/SC-07b, TC-016 and Task-018/#63 remain planned and retain their matrix/task owners. | TM-001; Task-018; IT-002 |

## Coverage

- Target allocation: `plan/Plan-001-finite-protocol-rust-slice/tasks/Task-021-complete-it002-replay-resource.md`.
- Task allocation: 1 / 1 done; all five implementation, evidence and promotion subtasks are complete.
- Scenario allocation: SC-05a..SC-05d and SC-06 are 5 / 5 passing; IT-002 overall is 10 / 12, with SC-07a/SC-07b still open under Task-018.
- Requirement trace: SC-05a/b/d exercise FR-041 and the FR-004-AC-4 aggregate; SC-05c verifies FR-026-AC-2; SC-06 verifies NFR-002. SC-07b remains the completion evidence for FR-041-AC-4's integration-wide refusal-axis comparison.
- Resource evidence: 7 / 7 active composed dimensions pass exact, one-short and fresh-exact probes after the separately budgeted producer-resolution boundary.
- Coordinate evidence: contract, schema digest, document identity/digest, authority identity/revision/digest, scope and population are 9 / 9 independently discriminated; four progress/closure axes, completeness and execution are 6 / 6 independently refused when cross-wired.
- Repository reconciliation: Quire reports 175 / 342 wider-plan rows backed; 263 / 270 Rust candidates are tagged and bound. Those planned gaps are outside this task-scoped verdict.
- Reverse surface: evidence matching, resource source sites, occurrence identity and model ordering all map to Task-021/IT-002 and their named FR/NFR owners; no copied owner decoder, hollow implementation, test-only production branch or unspecified new behavior was found.
- Semantic review: the optional gap-analysis semantic pass was not requested and was skipped. The required Rust review still checked the targeted intent, test and production-code alignment.

## Validation

The exact Cargo gate passes 284 unit/integration tests and 43 doc-tests, plus
formatting and all-target Clippy with warnings denied. Native Quoin
`0.23.1-87-gb3d02d4` strict validation and review-artifact validation report no
finding.

## Boundary

The full plan remains active. This artifact audits only Task-021 and neither
suppresses nor reclassifies IT-002 SC-07, Task-018/#63 or the later complete-V1
Protocol and temporal work.
