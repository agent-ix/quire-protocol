---
id: FR-051
title: "Link queue and channel declarations with capacity, key and overflow policy"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-044, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/ADR-027, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-050, type: references }
---
# FR-051: Link queue and channel declarations with capacity, key and overflow policy

## Description

When the linked protocol is built from an admitted checked package, the library SHALL retain each authored channel's delivery premise, ordering key, capacity and overflow policy exactly as the checked package declares them. The library SHALL mark a capacity that is not a literal as symbolic instead of resolving it.

## Inputs

An admitted checked package (FR-043) and the node id of a protocol clause.

## Outputs

Per channel: the checked node id, the delivery premise, the ordering (`fifo` with its key, or `unordered`), the capacity as a literal or as a symbolic expression, and the overflow policy (`block`, `reject`, or `loss` with its selector).

## Behavior

- The library shall obtain the delivery premise and capacity through QSL's `delivery` and `queue_capacity` accessors wherever they own the value (FR-044).
- The library shall keep a symbolic capacity as the authored expression.
- The library shall not evaluate it to a number.
- The library shall retain a channel that declares no overflow policy as having none.
- The library shall not supply one.
- Conflicting channel premises remain refused by FR-044; this requirement adds no second refusal for them.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-051-AC-1 | Over channels with a literal capacity, a symbolic capacity, each overflow policy and each ordering, the linked representation carries each field equal to the value the checked node and QSL's accessors report. | Test (TC-050) |
| FR-051-AC-2 | A symbolic capacity is carried as its authored expression, and no linked field holds a number derived from it. | Test (TC-050) |
| FR-051-AC-3 | A channel with no authored overflow policy is linked with no overflow policy and no default. | Test (TC-050) |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes what this library owns for dynamic roles, bounded queues, joins and loops, and what QSL owns.
- QSL ADR-027 states the step semantics this requirement does not restate; QSpec FR-170 and FR-171 state the thresholds and overflow rules the criteria cite.
