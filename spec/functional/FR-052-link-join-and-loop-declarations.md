---
id: FR-052
title: "Link join and loop declarations"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-044, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/ADR-027, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-050, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-051, type: references }
---
# FR-052: Link join and loop declarations

## Description

When the linked protocol is built from an admitted checked package, the library SHALL retain each authored join (its policy, its threshold and its outstanding-branch rule) and each authored loop (its iteration maximum and its declared invariant and variant) from the checked package, keyed to the control identities the checked package assigns.

## Inputs

An admitted checked package (FR-043) and the node id of a protocol clause.

## Outputs

Per join: the control identity, the policy (`all`, `any`, `quorum(n)` or `predicate`), the threshold and the `outstanding` rule or its absence. Per loop: the control identity, the maximum, and the invariant and variant declarations as authored.

## Behavior

- The library shall take every control identity from `protocol_view`.
- The library shall mint none (FR-044).
- The library shall carry a `predicate` join as a declared policy without evaluating its predicate.
- The library shall carry a loop's invariant and variant declarations as references to the checked nodes.
- The library shall not interpret them. Their discharge is not this library's (FR-059).
- The library shall carry a join with no `outstanding` rule as having none.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-052-AC-1 | Over joins of each policy and a quorum threshold, the linked representation carries the control identity, policy and threshold equal to the checked package's. | Test (TC-050) |
| FR-052-AC-2 | A `predicate` join and a join with no `outstanding` rule link successfully and are carried as declared, without evaluation and without a default rule. | Test (TC-050) |
| FR-052-AC-3 | A loop with an invariant and a lexicographic variant links with both declarations referenced by checked node id, and linking performs no check of either. | Test (TC-050) |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes what this library owns for dynamic roles, bounded queues, joins and loops, and what QSL owns.
- QSL ADR-027 states the step semantics this requirement does not restate; QSpec FR-170 and FR-171 state the thresholds and overflow rules the criteria cite.
