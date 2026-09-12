---
id: FR-004
title: "Assess finite global conformance"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-002
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-003
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-026
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-027
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-028
    type: contains
  - target: ix://agent-ix/quire-protocol/FR-041
    type: contains
  - target: ix://agent-ix/quire-specification/FR-059
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-220
    type: references
  - target: ix://agent-ix/quire-specification/FR-230
    type: references
  - target: ix://agent-ix/quire-specification/FR-240
    type: references
  - target: ix://agent-ix/quire-specification/FR-241
    type: references
  - target: ix://agent-ix/quire-specification/FR-247
    type: references
  - target: ix://agent-ix/quire-specification/FR-260
    type: references
---
# FR-004: Assess finite global conformance

## Description

When finite global conformance is requested, the evaluator SHALL assess the exact admitted observation history against the linked protocol's exact causal, channel, activation, compensation, decision-scope progress/closure, surrounding-execution progress/closure, completeness and complete-global-conformance premises through the atomic requirements FR-026, FR-027, FR-028 and FR-041.
When any one of FR-026, FR-027, FR-028 and FR-041 is unsatisfied, the evaluator SHALL report global conformance as unavailable rather than as an aggregate success.

## Inputs

A linked finite protocol, bound observation history, exact assessment request,
independent decision-scope and surrounding-execution progress/closure facts, the
complete-global-conformance closure record, resource budget, and evaluator
identity.

## Outputs

Per-obligation and global results with exact satisfied, violated, pending,
incomplete, refused, failed, unsupported, or exhausted causes.

## Behavior

This requirement is the aggregate. It establishes no obligation of its own beyond
composition: each obligation it previously stated in prose is now stated, once
and independently verifiably, by exactly one of its parts.

| Part | Obligation it owns |
| --- | --- |
| [FR-026](./FR-026-admit-every-permitted-observation-order.md) | Every presentation order the authored partial order and the selected channel premises permit yields the same result, and no causal edge is derived from a timestamp, an arrival position, an ingestion order or a key order. |
| [FR-027](./FR-027-report-adverse-observations-without-repair.md) | Unexpected, missing, misordered, duplicated and mis-correlated observations are reported against the exact affected obligation and never repaired. |
| [FR-028](./FR-028-keep-progress-and-closure-axes-independent.md) | The five progress/closure axes stay independent, and complete global success is withheld until the global closure state and every exact required premise hold. |
| [FR-041](./FR-041-agree-across-batch-and-incremental-paths.md) | Batch replay and settled incremental assessment agree value by value, a value settles at the earliest prefix that decides it, and a divergence between the paths refuses rather than reporting agreement. |

The four parts are separate requirements because their failures are separate
failures: a result can be order-dependent while diagnosing adversity correctly,
can diagnose adversity while substituting one closure axis for another, and can
keep the axes independent while agreeing between paths only vacuously. FR-026
and FR-041 are two parts rather than one for the same reason: order invariance is
a property of one path under a permuted input, path agreement is a property of
two paths on one input, and an evaluator can hold either without the other. No
part can be inferred from another, and the aggregate composes no success from the
parts that did hold.

FR-028 and FR-041 together carry what this requirement previously stated as one
sentence about complete global success: closing a decision scope closes neither
the surrounding execution nor global conformance (FR-028), and the equality of
batch and settled-incremental assessment is a property of the admitted facts
rather than of their arrival (FR-041).

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-004-AC-1 | Order invariance over the whole assessment is exactly what [FR-026](./FR-026-admit-every-permitted-observation-order.md) establishes: every permitted split-shipment and compensation interleaving produces the same conformant obligation set, and no other path reorders or re-presents the admitted history to obtain one. | Test (TC-004) |
| FR-004-AC-2 | Adverse reporting over the whole assessment is exactly what [FR-027](./FR-027-report-adverse-observations-without-repair.md) produces: unexpected, missing, causally misordered, duplicate-effect, and cross-workflow observations identify the exact affected obligation, and no aggregate path repairs one to avoid reporting it. | Test (TC-004) |
| FR-004-AC-3 | Axis independence over the whole assessment is exactly what [FR-028](./FR-028-keep-progress-and-closure-axes-independent.md) enforces: decision-scope progress/closure, surrounding-execution progress/closure and complete-global-conformance closure remain independent, changing or closing one cannot close another, and open, incomplete, unsupported, failed or exhausted assessments cannot become complete global success. | Test (TC-004) |
| FR-004-AC-4 | Path equivalence over the whole assessment is exactly what [FR-041](./FR-041-agree-across-batch-and-incremental-paths.md) establishes: batch replay and settled incremental assessment are semantically identical for every permutation of the same admitted facts, the incremental path settles a value strictly before the final fact on a history whose earlier prefix already contains every fact that value requires to be decided, and a divergence refuses rather than reporting agreement. | Test (TC-004) |

## Adopted vocabulary

Every noun below is typed upstream and adopted by reference. Each part carries
the slice of this table it actually uses; the table is retained here as the
shared mapping the decomposition was derived from, not as a second definition.

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Authored causal (partial) order | `FR-220-causal-control-graph` | derived |
| Protocol obligation | `FR-230-protocol-obligation` | derived, keyed (clause subject, semantic trigger-event identity) |
| Canonical protocol result | `FR-240-canonical-protocol-result` | not established upstream |
| Assessment-execution disposition | `FR-241-assessment-execution-disposition` | value |
| Complete-global-conformance closure state | `FR-247-complete-global-conformance-closure-state` | value |
| Admitted observation | `FR-260-admitted-observation` | not established upstream |

`FR-241`'s five members — `completed`, `refused`, `unsupported`,
`resource-incomplete`, `failed` — are exactly the non-success causes this
requirement's Outputs previously listed in prose, and the Outputs list is now a
reference to them rather than a second enumeration. `FR-247` supplies the fifth
closure axis and the `not-required` rule. The parts carry the rest.

### Where the upstream vocabulary does not yet reach

The gaps are recorded once, in the part that depends on each:

- `FR-026` records that an admitted fact set has no upstream key, that the
  equal-time tie key exists only as a refusal and depends on an `unresolved`
  temporal producer, that the channel premise key carries an undefined "key
  scope", and that the authored graph key is not stated to be complete.
- `FR-041` records that an admitted fact set has no upstream key, that the four
  progress/closure axes have no member list, that settlement-basis membership is
  not stated uniformly, and that the canonical result's own identity is
  undeclared.
- `FR-027` records that admitted observation, delivery and business effect have
  no key or order, that "duplicated" has two irreconcilable readings upstream,
  that a duplicate business effect is undetectable under its declared key and
  that its disposition is contradicted, and that cross-workflow correlation
  rests on an unfixed endpoint key.
- `FR-028` records that four of the five axes have no member list anywhere, that
  the canonical result's own identity is undeclared, and that closure at a
  deadline crosses an unresolved half-open/inclusive boundary conflict.

Every one of those is owned by `agent-ix/quire-specification`. None is supplied
locally.

## Dependencies

- [FR-002](./FR-002-link-finite-protocol-representation.md).
- [FR-003](./FR-003-assess-compensation-state.md).
- `ix://agent-ix/quire-specification/FR-059`.
- Parts: [FR-026](./FR-026-admit-every-permitted-observation-order.md),
  [FR-027](./FR-027-report-adverse-observations-without-repair.md),
  [FR-028](./FR-028-keep-progress-and-closure-axes-independent.md),
  [FR-041](./FR-041-agree-across-batch-and-incremental-paths.md).
