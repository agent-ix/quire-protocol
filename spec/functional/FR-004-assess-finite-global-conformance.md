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
  - target: ix://agent-ix/quire-specification/FR-059
    type: depends_on
---
# FR-004: Assess finite global conformance

## Description

When finite global conformance is requested, the evaluator SHALL assess the exact
admitted observation history against the linked protocol's causal, channel,
activation, compensation, decision-scope progress/closure,
surrounding-execution progress/closure, completeness, and declared
complete-global-conformance premises.

## Inputs

A linked finite protocol, bound observation history, exact assessment request,
independent decision-scope and surrounding-execution progress/closure facts,
resource budget, and evaluator identity.

## Outputs

Per-obligation and global results with exact satisfied, violated, pending,
incomplete, refused, failed, unsupported, or exhausted causes.

## Behavior

The evaluator SHALL accept every observation order permitted by the authored
partial order. It SHALL report unexpected, missing, misordered, duplicated, and
mis-correlated observations without repairing them. The evaluator SHALL NOT
report complete global success until the selected complete-global-conformance
closure and completeness premises hold. Closing a decision scope SHALL NOT close
the surrounding execution or establish complete global conformance. Batch replay
and settled incremental assessment over identical semantic inputs SHALL produce
the same result.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-004-AC-1 | Every permitted split-shipment and compensation interleaving produces the same conformant obligation set. | Test (TC-004) |
| FR-004-AC-2 | Unexpected, missing, causally misordered, duplicate-effect, and cross-workflow observations identify the exact affected obligation. | Test (TC-004) |
| FR-004-AC-3 | Decision-scope progress/closure, surrounding-execution progress/closure and complete-global-conformance closure remain independent; changing or closing one cannot close another, and open, incomplete, unsupported, failed or exhausted assessments cannot become complete global success. | Test (TC-004) |
| FR-004-AC-4 | Batch replay and settled incremental assessment are semantically identical for every permutation of the same admitted facts. | Test (TC-004) |

## Dependencies

- [FR-002](./FR-002-link-finite-protocol-representation.md).
- [FR-003](./FR-003-assess-compensation-state.md).
- `ix://agent-ix/quire-specification/FR-059`.
