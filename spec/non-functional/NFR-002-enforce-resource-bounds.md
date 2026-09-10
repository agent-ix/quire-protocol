---
id: NFR-002
title: "Enforce caller-supplied resource bounds"
type: NFR
quality_attribute: performance_efficiency
relationships:
  - target: ix://agent-ix/quire-protocol/FR-001
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-002
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-004
    type: constrains
---
# NFR-002: Enforce caller-supplied resource bounds

## Statement

The library SHALL enforce every positive caller-supplied input-byte, node, edge,
branch, iteration, observation, lineage, output-byte, operation, accounted-allocation,
and elapsed-time limit selected for the exact request according to the boundary
rules in this requirement.

## Scope

Applies to strict reading, linking, compensation assessment, finite replay,
result construction, support-lineage validation, and E02 projection.

Count, byte, operation, and accounted-allocation maxima are inclusive. The
library checks the requested increment before allocating, decoding, inserting,
or starting the next unit of work and refuses that unit if it would exceed its
maximum. Elapsed time uses a monotonic deadline checked before and after each
declared indivisible unit. The library starts no later unit after the deadline;
it returns resource-incomplete with the observed elapsed time and any indivisible-
unit overshoot. Engineering Assurance owns whole-process containment and external
resource enforcement.

## Rationale

Finite syntax alone does not prevent adversarial size or combinatorial cost.
The owner supplies budgets; the implementation must enforce them without
inventing a universal threshold.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
| --- | --- | --- | --- |
| Inclusive count/byte/operation/accounted-allocation limits with below/at/over controls | Every declared static dimension | 100% boundary discrimination | Test (TC-009) |
| Monotonic deadline checks before and after each indivisible unit | Every declared unit boundary | 100% stop-state discrimination | Test (TC-009) |
| Work admitted after a limit is exhausted | None | 0 cases | Test (TC-009) |

## Verification

Boundary tests exercise each dimension immediately below, at, and above its
owner-supplied limit. Mutation tests remove each guard and require its assigned
bare-traced case to fail for the intended reason.

## Dependencies

- [FR-001](../functional/FR-001-admit-selected-protocol-profile.md).
- [FR-002](../functional/FR-002-link-finite-protocol-representation.md).
- [FR-004](../functional/FR-004-assess-finite-global-conformance.md).
