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
  - target: ix://agent-ix/quire-observation/FR-004
    type: references
  - target: ix://agent-ix/quire-observation/FR-006
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

The window is the closed interval `[lower, upper]` that the checked `await` authors (`within [lower, upper]`), counted from the instant of the admitted observation of the awaited `after` event of that waiting instance. QSL publishes the window only as these offsets, so a wait has no window until that observation is admitted.

## Inputs

The linked `await` control, the identity of the waiting instance, the admitted observation of the awaited `after` event with its QObs anchor (quire-observation FR-004, FR-006), the admitted observation history, the awaited scope's progress state (FR-284) with the boundary and clock it is closed through (FR-289), the input completeness (FR-295), and any bound constraint on the awaited message as QSL's slot outcome (FR-045). Progress and completeness are independent facts and neither stands in for the other (FR-028). The anchor instant is read from the admitted `after` observation; no caller supplies it separately. QSL publishes the `after` anchor of each checked `await` as `after = {kind, target}` with kind `event` or `compensation`, and its causal relation anchors the `await` at the exit of the `event` control or at the entry of the `compensation` template's forward body; this library reads that port into the linked `await` declaration. A QObs observation carries neither the checked node nor the workflow instance, so the caller attributes an admitted observation to that port and to a waiting instance when it offers it, as it does for a response observation. For a `compensation` anchor the attributed observation is the one at the forward body's entry.

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
The evaluator SHALL refuse a progress view whose boundary is not the wait's window or whose clock is not the wait's clock.
The evaluator SHALL take the anchor instant of a wait from the anchor of the admitted `after` observation as the QObs observation view reports it: the position for an event position, the sample index (not the epoch) for a fixed sample and the signed nanosecond instant for a timestamped event.
The evaluator SHALL read from QSL's causal relation the port at which a linked `await` is anchored.
The evaluator SHALL NOT select the response branch or the timeout branch for a wait whose `after` observation is not admitted.
While the `after` observation is not admitted, the evaluator SHALL report truth `unavailable` with execution `completed`, because the wait's activation is not `triggered` and truth `pending` requires activation `triggered` (QSpec FR-061, FR-242).
When the `after` observation is admitted after a response observation or an authority fact of the same wait, the evaluator SHALL judge those facts against the resulting window by their place in the admitted history.
When the `after` observation is admitted after the evaluator reported a result for the wait without it, the evaluator SHALL emit a superseding or invalidating result under FR-033 naming the earlier result, because that result rested on the absence of the `after` observation.
When a second observation is offered as the `after` event of one waiting instance, the evaluator SHALL refuse it and leave the wait's results unaltered.
The evaluator SHALL refuse an `after` observation whose anchor is not of the clock family of the wait's progress boundary.
The evaluator SHALL refuse an `after` observation whose clock is not the wait's clock.
The evaluator SHALL refuse an `after` observation whose window is not the boundary of a progress view already admitted for the wait.
The evaluator SHALL refuse a window whose bounds counted from the anchor do not fit the integer range of the progress boundary's clock family: non-negative 64-bit for event positions and fixed samples, signed 128-bit nanoseconds for timestamped events (FR-289).

## Open questions for the quire-specification owner

STD-657 carries both questions.

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
| FR-100-AC-11 | A progress view whose lower bound differs, whose upper bound differs, or whose clock differs from the wait's is refused. | Test (TC-154) |
| FR-100-AC-12 | A wait keyed by a control that QSL's occurrence schema does not declare as an `await`, or by a wrong number of repeat ordinals, is refused, and a workflow identity that QSL cannot key is refused. | Test (TC-154) |
| FR-100-AC-13 | A window whose bounds counted from the anchor fall outside the integer range of the progress boundary's clock family (non-negative 64-bit for event positions and fixed samples, signed 128-bit for timestamped events) is refused. | Test (TC-154) |
| FR-100-AC-14 | The window of a wait is counted from the anchor of its admitted `after` observation: with the anchor at 10 and `within [0,29]` a response at 39 selects the response branch and a response at 40 does not, and with the anchor at 0 a response at 39 does not. | Test (TC-154) |
| FR-100-AC-15 | With no admitted `after` observation, a response observation together with any combination of progress `open` or `closed` and completeness `complete`, `incomplete` or `contradicted` selects neither branch and reports truth `unavailable` with execution `completed`, never `pending`. | Test (TC-154) |
| FR-100-AC-16 | An `after` observation admitted after the response observation and the authority facts yields a FR-033 superseding result naming the earlier result, which stays unaltered, and the batch settles to the same results as the incremental run. | Test (TC-154) |
| FR-100-AC-17 | A second `after` observation offered for one waiting instance is refused and leaves the earlier results unaltered. | Test (TC-154) |
| FR-100-AC-18 | An `after` observation whose anchor is not of the progress boundary's clock family is refused. | Test (TC-154) |
| FR-100-AC-19 | An `after` observation on another clock of the same clock family is refused. | Test (TC-154) |
| FR-100-AC-20 | A fixed-sample `after` observation counts the window from its sample index, not its epoch, and a window outside the unsigned 64-bit range of the sample index is refused. | Test (TC-154) |
| FR-100-AC-21 | Two waiting instances of one control each count their window from their own `after` observation. | Test (TC-154) |
| FR-100-AC-22 | An `after` observation whose window is not the boundary of an earlier admitted progress view is refused. | Test (TC-154) |
| FR-100-AC-23 | The linked `await` declaration names the port at which QSL's causal relation anchors the `await`. | Test (TC-154) |

## Dependencies

- FR-028 supplies the independent progress, closure and completeness axes this requirement reads.
- FR-033 supplies the superseding and invalidating result lineage used for a late observation and for a regressed premise.
- quire-observation FR-004 publishes the `quire.observation.record/v2` observation view whose anchor is the instant of the `after` event, and quire-observation FR-006 states the anchors of the three clock families.
- FR-045 supplies the slot outcome of a bound constraint, which this requirement reads and does not evaluate.
- QSpec FR-049 states the deadline branch; FR-241, FR-242, FR-284 and FR-295 define the dispositions and axes it is expressed in; FR-061 states that a contradicted premise is reported by supersession; FR-289 maps the native interval to the progress boundary.
