---
id: FR-056
title: "Assess an observed loop against its iteration bound"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-027, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-028, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-041, type: references }
  - { target: ix://agent-ix/quire-specification/FR-170, type: references }
  - { target: ix://agent-ix/quire-specification/FR-171, type: references }
  - { target: ix://agent-ix/quire-spec-language/ADR-027, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-052, type: depends_on }
---
# FR-056: Assess an observed loop against its iteration bound

## Description

When an admitted observation history is assessed against a linked protocol, the evaluator SHALL count each observed loop's iterations. The evaluator SHALL judge the count against the loop's authored maximum.

## Inputs

A linked protocol with its loop declarations (FR-052), and an admitted observation fact set, given to the assessment as the library's `LoopFact` values: each names the observation, the `repeat` control, the workflow instance, one repeat ordinal for each enclosing `repeat` and one of iteration or exit.

## Outputs

Per loop occurrence: the observed iteration count, whether it stayed within the authored maximum, and whether the loop exited; plus the typed adverse observations.

## Behavior

- The evaluator shall report an iteration count above the authored maximum as a loop-bound violation naming the loop, the maximum and the observed count.
- The evaluator shall report a loop whose history ends inside it as not exited.
- The evaluator shall not infer that it exits.
- For a loop with no authored maximum, the evaluator shall report the observed iteration count and the maximum as absent.
- For a loop with no authored maximum, the evaluator shall report no loop-bound violation and infer no maximum from the history.
- The evaluator shall keep one occurrence for each workflow instance and enclosing repeat ordinals, so a loop inside a `repeat` counts each outer iteration alone.
- The evaluator shall keep the exit axis separate from the closure axis (FR-028).
- The evaluator shall not evaluate the loop's invariant or variant against the history (FR-059).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-056-AC-1 | Over a loop with maximum m, histories with m-1, m and m+1 iterations report a loop-bound violation only for m+1. | Test (TC-054) |
| FR-056-AC-2 | A history that ends inside a loop reports the loop as not exited and reports no violation by that fact alone. | Test (TC-054) |
| FR-056-AC-3 | The loop assessment of a history is unchanged by the closure axis value assessed for the same history. | Test (TC-054) |
| FR-056-AC-4 | Over a loop with no authored maximum and a history of any iteration count, the assessment reports the observed count, reports the maximum as absent and reports no loop-bound violation. | Test (TC-054) |
| FR-056-AC-5 | Over a loop inside a `repeat`, iterations observed under two outer ordinals are two occurrences, each judged against the maximum alone, and an ordinal outside the outer repeat's maximum refuses the history. | Test (TC-054) |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes what this library owns for dynamic roles, bounded queues, joins and loops, and what QSL owns.
- QSL ADR-027 states the step semantics this requirement does not restate; QSpec FR-170 and FR-171 state the thresholds and overflow rules the criteria cite.
