---
id: FR-050
title: "Link role declarations without minting identity"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-044, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/ADR-027, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-051, type: references }
---
# FR-050: Link role declarations without minting identity

## Description

When the linked protocol is built from an admitted checked package, the library SHALL retain each authored role declaration (its lifetime kind, its population bound, its activation rule and its authored spawn and retire controls) as read from the checked package. The library SHALL mint no role instance identity of its own.

## Inputs

An admitted checked package (FR-043) and the node id of a protocol clause, read through the typed read view of FR-044.

## Outputs

The linked representation carrying, per role, the checked node id, the lifetime kind, the population bound or its absence, and the activation rule; or the typed `LinkRefusal` of FR-044.

## Behavior

- The library shall read role lifetime, population bound and activation rule from the checked node content and QSL's owner accessors.
- The library shall keep no table of its own.
- The library shall retain an absent population bound as absent.
- The library shall not substitute a default bound.
- The library shall assign no instance identity at link time. A role instance identity exists only where an observation in the history names it (FR-053).
- The library shall retain a role the checked package marks replicated as replicated.
- The library shall not expand it into a finite set of roles.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-050-AC-1 | Over a checked package declaring one role per lifetime kind (`workflow`, `scope`, `until(p)`), the linked representation lists each role with the checked node id and the lifetime kind that node carries. | Test (TC-050) |
| FR-050-AC-2 | Over a replicated role with a `max` bound and one with none, the linked representation carries the bound exactly and carries absence as absence, with no default. | Test (TC-050) |
| FR-050-AC-3 | Linking the same checked package twice yields equal representations, and no function of the crate generates a role instance identifier at link time. | Test (TC-050) |
| FR-050-AC-4 | A replicated role is linked as one declaration; the representation holds no per-instance expansion of it. | Test (TC-050) |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes what this library owns for dynamic roles, bounded queues, joins and loops, and what QSL owns.
- QSL ADR-027 states the step semantics this requirement does not restate; QSpec FR-170 and FR-171 state the thresholds and overflow rules the criteria cite.
