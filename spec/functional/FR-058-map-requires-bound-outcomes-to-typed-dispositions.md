---
id: FR-058
title: "Map QSL requires-bound outcomes to typed dispositions"
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
---
# FR-058: Map QSL requires-bound outcomes to typed dispositions

## Description

When QSL reports a requires-bound outcome for a protocol construct whose bound is missing, the library SHALL return a typed non-success disposition naming the missing bound. The library SHALL NOT truncate the construct at a finite bound to finish.

## Inputs

A QSL requires-bound outcome naming the construct, and the request's premises.

## Outputs

A typed non-success disposition carrying the QSL outcome unchanged, the checked node id of the construct that lacks the bound, and the kind of bound missing (role population, queue capacity, loop maximum, or other as QSL names it).

## Behavior

- The library shall not pick, default or search for a bound.
- The library shall not complete the analysis over a bound it chose.
- The library shall report the disposition as non-success on its own axis.
- The library shall not fold it into conformance, progress or closure (FR-028).
- The library shall carry no partial result with the disposition that would read as complete.
- When the same request asks other claims, the library shall leave their dispositions unchanged (FR-029).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-058-AC-1 | Given a requires-bound outcome for a role without a population bound, a channel without a capacity and a loop without a maximum, each disposition is typed non-success, carries QSL's outcome unchanged and names the construct's checked node id and the missing bound kind. | Test (TC-055) |
| FR-058-AC-2 | The library calls no QSL analysis with a bound the library derived, and no disposition field holds a bound the library chose. | Test (TC-055) |
| FR-058-AC-3 | A requires-bound disposition on one claim leaves every other requested claim's disposition unchanged and changes none of the conformance, progress or closure axis values. | Test (TC-055) |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes what this library owns for dynamic roles, bounded queues, joins and loops, and what QSL owns.
- QSL ADR-027 states the step semantics this requirement does not restate; QSpec FR-170 and FR-171 state the thresholds and overflow rules the criteria cite.
