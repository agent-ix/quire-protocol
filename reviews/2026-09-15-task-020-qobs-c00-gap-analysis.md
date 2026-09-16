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
surface after the Rust-review remediation. The compatibility allocation is
complete and preserves Task-018's separate ownership of final scenario-record
composition.

## Verdict

**PASS for Task-020's allocation.** Every subtask and deliverable is present,
the focused IT-002 evidence executes, and no in-scope plan, trace, reverse-code
or stub gap remains. This subset verdict is not a PASS for all of IT-002,
Plan-001 or epic #14.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Resolved: the first implementation distributed producer metadata across six edit sites, permitting an enum variant to be silently omitted from the manifest inventory. The generated inventory now makes declaration and all metadata exhaustive while preserving a separate literal C00 test oracle. | `src/producer_contracts.rs::define_producer_contracts`; IT-002-SC-08 |
| FND-002 | low | Repository-wide coverage remains intentionally incomplete outside this subset. IT-002 SC-05 through SC-07 and the Quoin handoff retain their existing owners. | TM-001; Task-018; IT-003 |
| FND-003 | medium | Resolved during mainline reconciliation: later merges had reintroduced the superseded claim that Task-010 executed ten of twelve IT-002 units. The plan and matrix now credit only Task-010 SC-01..SC-04 plus Task-020 SC-08 (5 / 12), leaving SC-05..SC-07 planned. | Task-010; Task-020; IT-002; TM-001 |

## Coverage

- Target allocation: `plan/Plan-001-finite-protocol-rust-slice/tasks/Task-020-adopt-qobs-c00-contracts.md`.
- Task allocation done: 1 / 1; all six Task-020 subtasks and the stated deliverable are complete.
- Trace path: Task-020 verifies IT-002; IT-002 SC-08 and its traced Rust test exercise FR-009's exact producer selection and NFR-006's pin/rights records.
- Contract inventory: 19 / 19 required QObs/QSL contracts resolve; 15 / 15 QObs schema bytes match accepted digests; the five C00 additions have an independent literal contract/digest oracle.
- Unsupported boundary: repair and aggregate query are 2 / 2 distinct typed unsupported capabilities and create 0 manifest entries.
- Reverse surface: the dependency pin, accepted digest metadata and unsupported capability type all map to Task-020/IT-002; no copied owner schema, decoder, hollow source or test stub was found.
- Repository reconciliation: Quire reports 175 / 342 rows backed by tagged symbols across the wider active plan; those planned gaps are outside this task-scoped verdict.
- Semantic review: the optional gap-analysis semantic pass was skipped because it was not requested. The required independent Rust review still checked Task-020's requirement, test and code alignment.

## Validation

The changed SpecReview, plan and requirement artifacts validate with the active
Quire catalog except for the repository's pre-existing TM-001 column-model
diagnostic (`Status` versus `Coverage Status`) and unrelated legacy
architecture-evaluation artifact. Native Quoin validation/coverage regressions
are tracked in `agent-ix/quoin#541` and `agent-ix/quoin#538`; Rust Quire supplied
the scoped validation and coverage evidence.

## Boundary

The full plan remains active. This artifact audits only the Task-020
compatibility successor and neither suppresses nor reclassifies the wider
repository's planned work.
