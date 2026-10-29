---
id: FR-026
title: "Admit every permitted observation order"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-004
    type: part_of
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-011
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-059
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-216
    type: references
  - target: ix://agent-ix/quire-specification/FR-220
    type: references
  - target: ix://agent-ix/quire-specification/FR-221
    type: references
  - target: ix://agent-ix/quire-specification/FR-230
    type: references
  - target: ix://agent-ix/quire-specification/FR-250
    type: references
  - target: ix://agent-ix/quire-specification/FR-252
    type: references
  - target: ix://agent-ix/quire-specification/FR-260
    type: references
---
# FR-026: Admit every permitted observation order

## Description

When one admitted fact set is presented in any order the authored causal order and the selected channel premises permit, the evaluator SHALL produce the same per-obligation and global assessment results.

## Inputs

A linked finite protocol carrying its authored partial acyclic causal order and
its selected channel premises, one admitted observation fact set, the
presentation order that set arrived in, and the accepted temporal profile with
its clock binding.

## Outputs

Per-obligation and global assessment results that are identical across every
presentation order the authored order and channel premises permit.

## Behavior

The evaluator SHALL admit every observation order the authored partial order and the selected channel premises permit.
The evaluator SHALL NOT derive a causal edge from a timestamp, an arrival position, an ingestion order, or a key order over any adopted upstream object.
The evaluator SHALL NOT reject a permitted presentation order in order to obtain a determinate result.

Order invariance is a property of one assessment path under a permuted input. It
is not the agreement of two paths on one input, which is
[FR-041](./FR-041-agree-across-batch-and-incremental-paths.md)'s obligation, and
neither property implies the other.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-026-AC-1 | Every permitted split-shipment and compensation interleaving produces the same conformant obligation set. | Test (TC-004) |
| FR-026-AC-2 | Permuting equal-time arrivals under the time key the accepted temporal profile declares changes no assessed value, over a non-singleton permutation set. | Test (TC-004) |
| FR-026-AC-3 | No timestamp, arrival position, ingestion order, or key order over an adopted upstream object is cited as evidence of a causal edge. | Inspection |

## Adopted vocabulary

The order this requirement is invariant under is upstream's authored partial
order, and the premises that widen or narrow the admitted orders are upstream's
channel premises. This requirement defines neither.

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Authored causal (partial) order | `FR-220` | derived, keyed (linked protocol identity, authored branches, join rules, iteration maxima) |
| Node occurrence | `FR-221` | derived, keyed (static declaration path, workflow instance identity, iteration identity) |
| Channel premise (all five components) | `FR-216` | value |
| Protocol obligation | `FR-230` | derived, keyed (clause subject, semantic trigger-event identity) |
| Temporal semantic profile | `FR-250` | value, keyed (label, definition revision, digest) |
| Temporal clock binding | `FR-252` | value |
| Admitted observation | `FR-260` | derived under FR-296 |

`FR-220-AC-3` is adopted directly as this requirement's FR-026-AC-3: it requires
that no key order declared by that artifact or by any object it contains is cited
as evidence of a causal edge. `FR-216` is why the admitted-order set is not the
authored order alone — an ordering policy and a delivery cardinality are
authored channel premises, and under FR-216 a FIFO premise reaches only its own
declared channel and key scope, so no FIFO order across channels is inferred
here.

### Accepted owner rulings

QSpec #31/#40 fixes the five-component FR-216 premise, its exact FIFO binder and
typed expression scope, the complete FR-220 graph identity and FR-296 admitted-
observation identity. QSL's accepted native-temporal contract supplies the
typed equal-time order key and authority; FR-289 supplies the only admitted
half-open-to-inclusive boundary conversion. Missing, foreign or unrepresentable
keys/boundaries refuse. Permutation equivalence is therefore componentwise over
accepted identities and never inferred from ingestion order or timestamps.

## Dependencies

- [FR-004](./FR-004-assess-finite-global-conformance.md) is the parent this
  requirement is part of.
- [FR-002](./FR-002-link-finite-protocol-representation.md) supplies the linked
  protocol whose authored partial order and channel premises the permitted
  presentations are drawn from.
- [FR-041](./FR-041-agree-across-batch-and-incremental-paths.md) is the sibling
  that owns batch/incremental path agreement.
- `ix://agent-ix/quire-specification/FR-059` establishes that the evaluator
  accepts any observation order consistent with the authored partial order and
  channel premises.
- `IT-002` scenario SC-05c verifies this requirement's equal-time permutation at
  the integration boundary.
