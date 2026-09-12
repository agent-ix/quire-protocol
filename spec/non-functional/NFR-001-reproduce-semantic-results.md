---
id: NFR-001
title: "Reproduce semantic results"
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/quire-protocol/US-003
    type: derives_from
  - target: ix://agent-ix/quire-specification/NFR-021
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-001
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-002
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-003
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-004
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-005
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-006
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-007
    type: constrains
  - target: ix://agent-ix/quire-protocol/FR-008
    type: constrains
---
# NFR-001: Reproduce semantic results

## Statement

For identical source, accepted source-baseline/profile, model, compiled-protocol, request,
binding, input, observation, decision-scope progress/closure,
surrounding-execution progress/closure, complete-global-conformance closure,
producer, implementation, and backend identities, the library SHALL produce
byte-identical canonical JSON and identical ordered semantic outcomes regardless
of map insertion order, presentation-only order, process locale, allocation
layout, or permitted causally independent observation-arrival order.

## Scope

Applies to admitted representation identities, linked protocol graphs,
compensation and conformance results, canonical protocol results, and E02
projection identities.

Unordered semantic sets use their declared canonical ordering. Authored
sequences and causal order remain unchanged. A source or formatting change that
changes an admitted identity is a semantic mutation, not an equivalent
presentation.

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
mutations of every named semantic identity, recompute RFC 8785 identities, and
compare bytes, ordered typed outcomes, and complete provenance.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| NFR-001-AC-1 | Every permitted map-order, locale, allocation-layout and causally independent arrival-order permutation that preserves all admitted identities produces byte-identical canonical JSON, identical ordered semantic outcomes and complete identical provenance. | Test (TC-008) |
| NFR-001-AC-2 | Independently changing each named source, accepted source-baseline/profile, model, compiled-protocol, request, binding, input, observation, decision-scope progress/closure, surrounding-execution progress/closure, complete-global-conformance closure, producer, implementation or backend identity changes the affected result identity or returns a typed refusal. | Test (TC-008) |
| NFR-001-AC-3 | Unordered semantic sets use their declared canonical order, while authored sequences and causal order remain unchanged and an identity-changing source presentation is never treated as equivalent. | Test (TC-008) |

## Dependencies

- `ix://agent-ix/quire-specification/NFR-021` is the formal-clause source.
- [FR-001](../functional/FR-001-admit-selected-protocol-profile.md).
- [FR-002](../functional/FR-002-link-finite-protocol-representation.md).
- [FR-003](../functional/FR-003-assess-compensation-state.md).
- [FR-004](../functional/FR-004-assess-finite-global-conformance.md).
- [FR-005](../functional/FR-005-dispatch-independent-protocol-claims.md).
- [FR-006](../functional/FR-006-produce-canonical-protocol-results.md).
- [FR-007](../functional/FR-007-project-results-to-e02.md).
- [FR-008](../functional/FR-008-return-typed-refusals.md).
