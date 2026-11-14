---
id: FR-057
title: "Map QSL Unsupported outcomes to typed dispositions"
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
# FR-057: Map QSL Unsupported outcomes to typed dispositions

## Description

When QSL reports an `Unsupported` cause for a protocol construct, the library SHALL return a typed `unsupported` disposition that names the construct. The library SHALL NOT approximate, truncate or repair the construct.

## Inputs

A QSL `Unsupported` cause for a requested claim or an assessed construct, and the exact premises the request carried.

## Outputs

One `unsupported` disposition (FR-029) that carries the QSL cause name unchanged and the checked node id and source locus of the construct, bound to its exact fragment, backend and premises (FR-030).

## Behavior

- The library shall carry the QSL cause name unchanged.
- When the assessment of an observed history meets a construct for which QSL would report an `Unsupported` cause, QSL having reported none for that history, the library shall return the same disposition carrying the name of that QSL `Unsupported` variant (`SymbolicCapacity` or the other variants QSL defines).
- When the assessment of an observed history meets a `join predicate(..)` or an `unordered` channel whose overflow loses its `head` or `tail`, which QSL steps and this library's assessment does not, the library shall return an `unsupported` disposition whose cause is the library's own assessment cause (`JoinPredicate`, `UnorderedLossSelector`), with no QSL `Unsupported` variant behind it (ADR-010: QSL owns the engines).
- When a discharge or other claim is requested that no QSL engine serves, the library shall carry QSL's `unsupported-requested-capability` cause as the name.
- The library shall define no second name for it. The QSL causes ADR-010 records for the extension that QSL still reports are symbolic capacity and a step binder of an unenumerable record type; QSL steps the others, and refuses a non-`all` join with no `outstanding` rule at admission.
- The library shall report no `unsupported` disposition as false, satisfied, conformant or successful.
- The library shall derive no result from it.
- The library shall leave every other claim's disposition and every independently supported result unchanged (FR-029).
- The library shall substitute no finite bound, no sampled value, no default policy and no local approximation for the unsupported construct.
- The library shall carry a cause it does not recognise unchanged under the unknown-source rule of FR-008.
- The library shall not map it to a known cause.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-057-AC-1 | Given each QSL cause ADR-010 lists, the disposition is `unsupported`, carries that cause name unchanged and the construct's checked node id, and is bound to the request's exact premises. | Test (TC-055) |
| FR-057-AC-2 | Given an `unsupported` cause on one claim of a request of n claims, the other n-1 dispositions equal those returned without it, and the unsupported claim is neither success nor failure. | Test (TC-055) |
| FR-057-AC-3 | Given a symbolic-capacity channel and a `predicate` join, no disposition or assessment field holds a number or truth value derived from evaluating the capacity or predicate. | Test (TC-055) |
| FR-057-AC-4 | Given a QSL cause name the library does not recognise, the disposition retains it unchanged as an unknown source cause and is not mapped to a known cause. | Test (TC-055) |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes what this library owns for dynamic roles, bounded queues, joins and loops, and what QSL owns.
- QSL ADR-027 states the step semantics this requirement does not restate; QSpec FR-170 and FR-171 state the thresholds and overflow rules the criteria cite.
