---
id: FR-006
title: "Produce canonical protocol results"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/US-002
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-004
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-005
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-061
    type: depends_on
---
# FR-006: Produce canonical protocol results

## Description

When an assessment disposition is available, the result producer SHALL emit one
strict-reader-valid `quire.protocol.result/v1-draft.1` artifact that preserves
every orthogonal fact and exact identity required by source requirement FR-061.

## Inputs

One exact request, assessment dispositions, decision-scope progress/closure,
surrounding-execution progress/closure, completeness facts, assumptions,
decision support, adequacy, provenance, limitations, and applicable prior result.

## Outputs

One canonical protocol result plus its non-self-referential RFC 8785
`sha256-jcs` artifact reference, or one typed construction/strict-reader refusal.

## Behavior

The producer SHALL keep execution, decision-scope progress, decision-scope
closure, surrounding-execution progress, surrounding-execution closure, truth,
settlement basis, activation, participation, assumptions, completeness,
observation adequacy, protocol adequacy, claim, and provenance separate.
The strict reader SHALL enforce the complete FR-061 result-combination table,
recompute identities, and reject missing, cyclic, self-referential, or falsely
independent support.
When later input contradicts a retained premise, the producer SHALL emit a linked
superseding or invalidating result while preserving every byte and identity of
the earlier result.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-006-AC-1 | Every valid FR-061 result-combination row is accepted and every one-axis forbidden combination is refused. | Test (TC-006) |
| FR-006-AC-2 | Settled truth on an open subject requires exact complete decisive support whose truth is preserved by every admitted continuation. | Test (TC-006) |
| FR-006-AC-3 | Assumption, observation-adequacy, protocol-adequacy, execution, truth, activation, participation, decision-scope progress, decision-scope closure, surrounding-execution progress, surrounding-execution closure, settlement, claim, and completeness mutations remain independently observable. | Test (TC-006) |
| FR-006-AC-4 | Every source, profile, model, protocol, request, binding, input, backend, decision-scope progress/closure, surrounding-execution progress/closure, settlement, support, producer, and prior-result identity is recomputed or strictly matched. | Test (TC-006) |
| FR-006-AC-5 | Support lineage rejects missing predecessors, cycles, self-reference, and invented independence while retaining shared and derived relations. | Test (TC-006) |
| FR-006-AC-6 | A later contradiction emits a new linked result and cannot rewrite or reseal the earlier artifact. | Test (TC-006) |

## Dependencies

- [FR-004](./FR-004-assess-finite-global-conformance.md).
- [FR-005](./FR-005-dispatch-independent-protocol-claims.md).
- `ix://agent-ix/quire-specification/FR-061`.
