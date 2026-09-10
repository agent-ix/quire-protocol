---
id: NFR-001
title: "Reproduce semantic results"
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/quire-protocol/FR-001
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-004
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-006
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-007
    type: constrains
---
# NFR-001: Reproduce semantic results

## Statement

The library SHALL produce byte-identical canonical JSON and semantic outcomes
for identical complete semantic inputs regardless of map insertion order,
presentation order, process locale, or equivalent observation arrival order.

## Scope

Applies to admitted representation identities, linked protocol graphs,
compensation and conformance results, canonical protocol results, and E02
projection identities.

## Rationale

Qualification, replay, and independent consumers require rebuildable identities.
Presentation-dependent output would make equivalent runs incomparable and permit
identity substitution.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
| --- | --- | --- | --- |
| Exact-byte reproduction across declared equivalent permutations | All retained cases | 100% byte identity | Test (TC-008) |
| Semantic input mutations changing the affected identity or refusing | All declared one-axis mutations | 100% discrimination | Test (TC-008) |

## Verification

Property and metamorphic tests generate equivalent presentations and one-axis
semantic mutations, recompute RFC 8785 identities, and compare both bytes and
typed outcomes.

## Dependencies

- [FR-001](../functional/FR-001-admit-selected-protocol-profile.md).
- [FR-004](../functional/FR-004-assess-finite-global-conformance.md).
- [FR-006](../functional/FR-006-produce-canonical-protocol-results.md).
- [FR-007](../functional/FR-007-project-results-to-e02.md).
