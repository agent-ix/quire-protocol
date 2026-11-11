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
  - target: ix://agent-ix/quire-protocol/FR-045
    type: references
  - target: ix://agent-ix/quire-specification/FR-049
    type: references
  - target: ix://agent-ix/quire-specification/FR-241
    type: references
  - target: ix://agent-ix/quire-specification/FR-242
    type: references
  - target: ix://agent-ix/quire-specification/FR-061
    type: references
  - target: ix://agent-ix/quire-specification/FR-284
    type: references
  - target: ix://agent-ix/quire-specification/FR-289
    type: references
  - target: ix://agent-ix/quire-specification/FR-295
    type: references
---
# FR-100: Settle inclusive deadline branches only on complete progress

Closes V1-PROT-019 (inclusive timeout branches). Extends FR-028; no existing requirement states how a deadline branch is selected. The settlement vocabulary is not new: the selected branch is carried with the FR-241 execution disposition the owner already defines, and an unselected wait with the FR-242 truth disposition.

## Description

When the evaluator assesses a waiting instance of a protocol branch that has an authored deadline, the evaluator SHALL select the response branch for a matching observation inside the authored window, counting an observation at exactly the upper bound as inside the wait.

When the awaited scope's progress (FR-284) is `closed` through the deadline, the input completeness (FR-295) is `complete` and no observation inside the window was admitted, the evaluator SHALL select the timeout branch.

The window is the closed interval `[lower, upper]` that the checked `await` authors (`within [lower, upper]`), taken from the anchor instant of the awaited `after` event.

## Inputs

The linked `await` control, the identity of the waiting instance, the anchor instant, the admitted observation history, the awaited scope's progress state (FR-284) with the boundary it is closed through (FR-289), the input completeness (FR-295), and any bound constraint on the awaited message as QSL's slot outcome (FR-045). Progress and completeness are independent facts and neither stands in for the other (FR-028).

## Outputs

For each waiting instance, keyed by that instance and not by the static branch, the selected branch (response or timeout) with its FR-241 execution disposition, or no selection with its FR-241 execution disposition and, where this requirement assigns one, its FR-242 truth disposition.

## Behavior

The evaluator SHALL NOT select timeout when progress is `open` through the deadline.
The evaluator SHALL NOT select timeout when completeness is `incomplete` or `contradicted`.
The evaluator SHALL NOT select timeout when an observation inside the window has a bound constraint that is false or valueless.
A bound constraint is valueless when QSL's slot outcome for it is unavailable, undefined, refused, incomplete, a family result or a value that is not a Boolean (FR-045).
The evaluator SHALL NOT select the response branch for an observation before the window's lower bound or after its upper bound.
Two waits on the same static branch SHALL settle independently.
Absent selection is truth `pending` (FR-242) while the deadline is not reached or progress is `open`.
Absent selection under completeness `incomplete` or `contradicted` is truth `unavailable` (FR-242) with execution `completed` (FR-241).
When progress is `open` and completeness is `incomplete` or `contradicted` at once, the truth is `unavailable`, because `pending` requires complete observations (QSpec FR-061).
When completeness is `incomplete` or `contradicted`, the completeness axis carries the FR-295 value and the progress axis is unchanged.
When progress is `closed`, completeness is `complete` and an observation inside the window has a valueless bound constraint, the evaluator SHALL report truth `unavailable` with execution `completed`.
When progress is `closed`, completeness is `complete` and the only observations inside the window have a false bound constraint, the evaluator SHALL report execution `completed` and no truth disposition.
When an observation inside the window is admitted after timeout was selected, by arrival order in the admitted history, the evaluator SHALL emit a superseding or invalidating result under FR-033.
When progress or completeness is later admitted so that it no longer supports a selected timeout, the evaluator SHALL emit a superseding or invalidating result under FR-033.
The evaluator SHALL leave the earlier result unaltered.
The evaluator SHALL refuse a progress view whose boundary is not the wait's window.

## Open questions for the quire-specification owner

- QSpec FR-242 names no truth disposition for a wait whose progress is `closed`, whose completeness is `complete` and whose only observations have a false bound constraint. FR-049-AC-9 says only that timeout never substitutes for a false constraint. This requirement assigns no truth for that case and does not settle it locally.
- QSpec FR-242 and FR-061 do not state whether the selection of a response or timeout branch is an FR-061 result that carries a truth disposition. This requirement carries the selected branch with its execution disposition only and does not settle it locally.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-100-AC-1 | A matching response observed exactly at the upper bound selects the response branch, and the same response at a time strictly after it, with progress `closed` and completeness `complete`, selects timeout. | Test (TC-154) |
| FR-100-AC-2 | No response with progress `closed` through the deadline and completeness `complete` selects timeout; the same history with progress `open` is truth `pending`; with completeness `incomplete` or `contradicted` it is truth `unavailable` with execution `completed` and the completeness axis carrying `incomplete` or `contradicted`; with progress `open` and completeness `incomplete` or `contradicted` together it is truth `unavailable`, not `pending`; none selects timeout. | Test (TC-154) |
| FR-100-AC-3 | A response whose bound constraint is false does not select the response branch and does not by itself select timeout. | Test (TC-154) |
| FR-100-AC-4 | A matching response that arrives after timeout was selected, by position in the admitted history, but is observed inside the window, yields a FR-033 superseding or invalidating result naming the earlier result and leaves its bytes and identity unchanged; a response observed after the deadline yields no such result, and batch and incremental runs agree on both. | Test (TC-154) |
| FR-100-AC-5 | Two waiting instances of one static branch, one answered and one not, settle differently and independently. | Test (TC-154) |
| FR-100-AC-6 | No business operation is invoked on any settlement path. | Test (TC-154) |
| FR-100-AC-7 | With progress `closed` and completeness `complete`, a false bound constraint inside the window selects no branch, reports execution `completed` and no truth disposition, and a valueless bound constraint selects no branch and reports truth `unavailable` with execution `completed`. | Test (TC-154) |
| FR-100-AC-8 | A false or valueless observation inside the window that is admitted after timeout was selected yields a FR-033 superseding or invalidating result naming the earlier result, and the earlier result is unaltered. | Test (TC-154) |
| FR-100-AC-9 | A progress or completeness fact admitted after timeout was selected that reports progress `open`, or completeness `incomplete` or `contradicted`, yields a FR-033 superseding or invalidating result naming the earlier result, and the batch settles to the same results as the incremental run. | Test (TC-154) |
| FR-100-AC-10 | A matching response observed before the window's lower bound does not select the response branch, and with progress `closed` and completeness `complete` timeout is selected. | Test (TC-154) |
| FR-100-AC-11 | A progress view whose boundary is not the wait's window is refused. | Test (TC-154) |
| FR-100-AC-12 | A wait keyed by a control that QSL's occurrence schema does not declare as an `await`, or by a wrong number of repeat ordinals, is refused, and a workflow identity that QSL cannot key is refused. | Test (TC-154) |

## Dependencies

- FR-028 supplies the independent progress, closure and completeness axes this requirement reads.
- FR-033 supplies the superseding and invalidating result lineage used for a late observation and for a regressed premise.
- FR-045 supplies the slot outcome of a bound constraint, which this requirement reads and does not evaluate.
- QSpec FR-049 states the deadline branch; FR-241, FR-242, FR-284 and FR-295 define the dispositions and axes it is expressed in; FR-061 states that a contradicted premise is reported by supersession; FR-289 maps the native interval to the progress boundary.
