---
id: FR-059
title: "Report loop invariant and variant discharge as unavailable, never as proved"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-005, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-029, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-030, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: references }
  - { target: ix://agent-ix/quire-spec-language/ADR-017, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/ADR-020, type: references }
  - { target: ix://agent-ix/quire-spec-language/ADR-027, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-052, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-056, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-057, type: depends_on }
---
# FR-059: Report loop invariant and variant discharge as unavailable, never as proved

## Description

While QSL provides no engine that discharges a `repeat`'s invariant or variant (lexicographic variants included), the library SHALL carry each declared invariant and variant as undischarged. The library SHALL return a typed `unsupported` disposition for any request for their proof.

## Inputs

A linked loop with declared invariant and variant (FR-052), and a request for their discharge.

## Outputs

The loop's declarations referenced by checked node id, a discharge status of `undischarged`, and, for a discharge request, an `unsupported` disposition (FR-057) that carries QSL's `unsupported-requested-capability` cause (FR-057).

## Behavior

- The library shall implement no invariant check, no variant ranking and no termination argument.
- The library shall not infer that a loop terminates, or that its invariant holds, from an observed history, from its authored maximum or from the progress check QSL reports.
- The library shall not report a bounded loop (FR-056) as proved.
- When QSL later serves the discharge, the library shall dispatch the request and carry the result under FR-005 and FR-030, and this requirement ceases to apply to the kinds QSL serves.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-059-AC-1 | Over a loop with an invariant and a lexicographic variant, the result carries both by checked node id with discharge status `undischarged`. | Test (TC-054) |
| FR-059-AC-2 | A request for the discharge of that loop returns an `unsupported` disposition naming the missing capability, bound to the request's premises, and no proved status. | Test (TC-054) |
| FR-059-AC-3 | Over a history in which the loop ends within its maximum and exits, the loop's discharge status remains `undischarged` and the loop bound assessment (FR-056) reports no proof. | Test (TC-054) |
| FR-059-AC-4 | Over a loop with an invariant and a variant, no result or disposition field reports an invariant checked, a variant rank or a termination verdict, and the only discharge status produced is `undischarged`. | Test (TC-054) |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes what this library owns for dynamic roles, bounded queues, joins and loops, and what QSL owns.
- QSL ADR-027 states the step semantics this requirement does not restate; QSpec FR-170 and FR-171 state the thresholds and overflow rules the criteria cite.
