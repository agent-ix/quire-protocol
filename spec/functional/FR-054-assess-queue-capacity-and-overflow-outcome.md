---
id: FR-054
title: "Assess an observed history against queue capacity and overflow outcome"
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
  - { target: ix://agent-ix/quire-protocol/FR-051, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-057, type: depends_on }
---
# FR-054: Assess an observed history against queue capacity and overflow outcome

## Description

When an admitted observation history is assessed against a linked protocol, the evaluator SHALL judge each channel's observed occupancy against its authored literal capacity. The evaluator SHALL judge each observed overflow event against the channel's authored overflow policy.

## Inputs

A linked protocol with its channel declarations (FR-051), and an admitted observation fact set.

## Outputs

Per channel: the observed maximum occupancy, whether it stayed within capacity, and, for each overflow event, whether the observed outcome (blocked send, rejection, or loss of the head, tail or new message) matches the authored policy; plus the typed adverse observations.

## Behavior

- The evaluator shall judge occupancy only against a literal capacity. For a symbolic capacity it shall return the disposition of FR-057.
- The evaluator shall not evaluate the expression.
- The evaluator shall report occupancy above capacity with no authored overflow event as a capacity violation naming the channel, capacity and observed occupancy.
- The evaluator shall report an overflow outcome that differs from the authored policy as an overflow violation naming the channel, the authored policy and the observed outcome.
- The evaluator shall judge delivery order against the authored ordering key and report an order the key forbids as an order violation.
- For an `unordered` channel the evaluator shall report no order violation for any delivery order.
- The evaluator shall report an observed loss, duplication or reordering that the channel's delivery premise does not permit as a premise violation naming the channel and the premise.
- For an `unordered` channel whose overflow loses its `head` or `tail`, the evaluator shall return the disposition of FR-057 for that channel.
- The evaluator shall report each violation without repairing it (FR-027).
- This requirement states no step semantics. QSL owns the exploration semantics (QSL ADR-027); this library assesses one admitted observation history (ADR-010 Decision 2).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-054-AC-1 | Over a literal-capacity channel whose history stays within capacity, the assessment reports no capacity violation and the observed maximum occupancy. | Test (TC-052) |
| FR-054-AC-2 | Over histories whose occupancy exceeds capacity with and without an authored overflow event, the assessment reports a capacity violation only for the one without. | Test (TC-052) |
| FR-054-AC-3 | Over one history per authored policy (`block`, `reject`, `loss head`, `loss tail` and `loss new` on `fifo` channels, `loss new` on an `unordered` channel) whose observed outcome matches, and one whose outcome differs, the assessment reports no violation for each match and an overflow violation naming authored policy and observed outcome for the mismatch. | Test (TC-052) |
| FR-054-AC-4 | Over a `fifo` channel keyed by a field, a history delivering two messages of one key out of send order yields an order violation, and a history interleaving different keys does not. | Test (TC-052) |
| FR-054-AC-5 | Over a channel with a symbolic capacity, the assessment returns the FR-057 disposition and reports neither a capacity violation nor its absence. | Test (TC-052) |
| FR-054-AC-6 | Over a channel whose delivery premise forbids loss and a history with a lost message, and over one whose premise permits it, the assessment reports a premise violation naming channel and premise only for the first. | Test (TC-052) |
| FR-054-AC-7 | Over an `unordered` channel, any delivery order of the same messages reports no order violation; over an `unordered` channel whose overflow loses `head` or `tail`, the assessment returns the FR-057 disposition for that channel. | Test (TC-052) |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes what this library owns for dynamic roles, bounded queues, joins and loops, and what QSL owns.
- QSL ADR-027 states the step semantics this requirement does not restate; QSpec FR-170 and FR-171 state the thresholds and overflow rules the criteria cite.
