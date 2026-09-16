---
id: SR-128
title: "Gap analysis of the Task-020 QObs C00 consumer successor"
type: SpecReview
analysis: gap-analysis
scope: "Task-020 / quire-protocol#12, IT-002 SC-08 and the QObs C00 compatibility allocation"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-020, type: reviews }
  - { target: ix://agent-ix/quire-protocol/IT-002, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: references }
---
# SR-128: Gap Analysis of the Task-020 QObs C00 Consumer Successor

## Summary

Audited Task-020's plan state, IT-002 trace, dependency records and production
surface after the Rust-review remediation and accepted-owner repin. The
implementation allocation is complete and preserves Task-018's separate
ownership of final scenario-record composition.

## Verdict

**PASS for Task-020 promotion.** The focused IT-002 evidence and implementation
are complete, and Protocol now pins accepted QObs PR #27 merge `9240063`.
This subset verdict is not a verdict on all of IT-002, Plan-001 or epic #14.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Resolved: the first implementation distributed producer metadata across six edit sites, permitting an enum variant to be silently omitted from the manifest inventory. The generated inventory now makes declaration and all metadata exhaustive while preserving a separate literal oracle for all twenty-one scenario contracts. | `src/producer_contracts.rs::define_producer_contracts`; IT-002-SC-08 |
| FND-002 | low | Repository-wide coverage remains intentionally incomplete outside this subset. IT-002 SC-05 through SC-07 and the Quoin handoff retain their existing owners. | TM-001; Task-018; IT-003 |
| FND-003 | medium | Resolved during mainline reconciliation: later merges had reintroduced the superseded claim that Task-010 executed ten of twelve IT-002 units. The plan and matrix now credit only Task-010 SC-01..SC-04 plus Task-020 SC-08 (5 / 12), leaving SC-05..SC-07 planned. | Task-010; Task-020; IT-002; TM-001 |
| FND-004 | high | Resolved in code: the standalone resolver did not guard the real Task-010 assessment path. Public assessment entry points now resolve before invoking their owner-input loaders, compare a fixed-size package/static/temporal request selection, and privately construct `ResolvedAssessmentEngine`; unresolved/repointed states return typed unexecutable without invoking the loader. | `src/assessment.rs`; `src/producer_contracts.rs`; TC-002; TC-003; TC-006 |
| FND-005 | medium | Resolved in evidence: SC-08 retains and asserts exact affected identity, non-authoritative sidecar irrelevance and one-entry repin independence; TC-002 proves zero owner reads on unresolved input, one load after resolution, and refusal when a resolution is retargeted to another temporal request. | IT-002-SC-08; TC-002; FR-009-AC-2..AC-5; FR-009-AC-8 |
| FND-006 | high | Resolved: QObs PR #27 merged as `924006300f45b38483be1cbdf99b68f899b7d368`; Cargo, lockfile, source constant, tests, dependency record, plan and matrix now select that accepted revision, and the final promotion subtask is complete. | Task-020; TM-001; `DEPENDENCIES.md`; QObs PR #27 |

## Coverage

- Target allocation: `plan/Plan-001-finite-protocol-rust-slice/tasks/Task-020-adopt-qobs-c00-contracts.md`.
- Task allocation: 1 / 1 done; all six implementation, evidence and promotion subtasks are complete.
- Trace path: Task-020 verifies IT-002; IT-002 SC-08 and its traced Rust test exercise FR-009's exact producer selection and NFR-006's pin/rights records.
- Contract inventory: 21 / 21 required QObs/QSL contracts resolve locally, including compiled `/2` and activation mapping `/3`; 15 / 15 QObs schema bytes match the exact accepted-revision digests; the complete owner/contract/revision population has an independent literal oracle.
- Unsupported boundary: repair and aggregate query are 2 / 2 distinct typed unsupported capabilities and create 0 manifest entries.
- Reverse surface: the dependency pin, accepted-revision digest metadata, enforced assessment wrapper and unsupported capability type all map to Task-020/IT-002; no copied owner schema, decoder, hollow source or test stub was found.
- Repository reconciliation: Quire reports 175 / 342 rows backed by tagged symbols across the wider active plan; those planned gaps are outside this task-scoped verdict.
- Semantic review: the optional gap-analysis semantic pass was skipped because it was not requested. The required independent Rust review still checked Task-020's requirement, test and code alignment.

## Validation

Both changed root SpecReviews and the Task-020 plan artifacts validate with the
active Quire catalog. Repository-wide Quire validation still reports only the
pre-existing TM-001 column-model diagnostic (`Status` versus `Coverage Status`)
and unrelated legacy architecture-evaluation analysis value. Native Quoin
0.23.1 validation returns zero findings. Authoritative Quire coverage remains
175 / 342 backed rows with 259 / 266 Rust candidates tagged and bound.

## Boundary

The full plan remains active. This artifact audits only the Task-020
compatibility successor and neither suppresses nor reclassifies the wider
repository's planned work.
