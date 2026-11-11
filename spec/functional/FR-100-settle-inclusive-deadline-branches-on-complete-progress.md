---
id: FR-100
title: "Settle inclusive deadline branches only on complete progress"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/ADR-010
    type: references
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-028
    type: extends
  - target: ix://agent-ix/quire-protocol/FR-019
    type: references
  - target: ix://agent-ix/quire-protocol/FR-033
    type: references
  - target: ix://agent-ix/quire-specification/FR-049
    type: references
  - target: ix://agent-ix/quire-specification/FR-241
    type: references
  - target: ix://agent-ix/quire-specification/FR-242
    type: references
  - target: ix://agent-ix/quire-specification/FR-284
    type: references
  - target: ix://agent-ix/quire-specification/FR-295
    type: references
---
# FR-100: Settle inclusive deadline branches only on complete progress

Closes V1-PROT-019 (inclusive timeout branches). Extends FR-028; no existing requirement states how a deadline branch is selected. The settlement vocabulary is not new: the selected branch is carried with the FR-242 truth disposition and FR-241 execution disposition the owner already defines.

## Description

When the evaluator assesses a waiting instance of a protocol branch that has an authored deadline, the evaluator SHALL select the response branch for a matching observation at or before the deadline, counting an observation at exactly the deadline as inside the wait.

When the awaited scope's progress (FR-284) is `closed` through the deadline and the input completeness (FR-295) is `complete`, and no matching observation was admitted, the evaluator SHALL select the timeout branch.

## Inputs

The linked deadline branch, the identity of the waiting instance, the admitted observation history, the awaited scope's progress state (FR-284), the input completeness (FR-295), and any bound constraint on the awaited message. Progress and completeness are independent facts and neither stands in for the other (FR-028).

## Outputs

For each waiting instance, keyed by that instance and not by the static branch, the selected branch (response or timeout) with its FR-242 truth disposition and FR-241 execution disposition, or no selection.

## Behavior

The evaluator SHALL NOT select timeout when progress is `open` through the deadline, when completeness is `incomplete` or `contradicted`, or when the only observation has a false bound constraint.
Two waits on the same static branch SHALL settle independently.
Absent selection is truth `pending` (FR-242) while the deadline is not reached or progress is `open`.
Absent selection under completeness `incomplete` or `contradicted` is truth `unavailable` (FR-242) with execution `completed` (FR-241).
In both of those cases the completeness axis carries the FR-295 value, `incomplete` or `contradicted`, and the progress axis is unchanged.
When an observation observed at or before the deadline is admitted after timeout was selected, by arrival order in the admitted history, the evaluator SHALL emit a superseding or invalidating result under FR-033.
The evaluator SHALL leave the earlier result unaltered.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-100-AC-1 | A matching response observed exactly at the deadline instant selects the response branch, and the same response at a time strictly after the deadline, with progress `closed` and completeness `complete`, selects timeout. | Test (TC-154) |
| FR-100-AC-2 | No response with progress `closed` through the deadline and completeness `complete` selects timeout; the same history with progress `open` is truth `pending`; with completeness `incomplete` or `contradicted` it is truth `unavailable` with execution `completed` and the completeness axis carrying `incomplete` or `contradicted`; none selects timeout. | Test (TC-154) |
| FR-100-AC-3 | A response whose bound constraint is false does not select the response branch and does not by itself select timeout. | Test (TC-154) |
| FR-100-AC-4 | A matching response that arrives after timeout was selected, by position in the admitted history, but is observed at or before the deadline, yields a FR-033 superseding or invalidating result naming the earlier result and leaves its bytes and identity unchanged; a response observed after the deadline yields no such result, and batch and incremental runs agree on both. | Test (TC-154) |
| FR-100-AC-5 | Two waiting instances of one static branch, one answered and one not, settle differently and independently. | Test (TC-154) |
| FR-100-AC-6 | No business operation is invoked on any settlement path. | Test (TC-154) |

## Dependencies

- FR-028 supplies the independent progress, closure and completeness axes this requirement reads.
- FR-033 supplies the superseding and invalidating result lineage used for a late observation.
- QSpec FR-049 states the deadline branch; FR-241, FR-242, FR-284 and FR-295 define the dispositions and axes it is expressed in.
