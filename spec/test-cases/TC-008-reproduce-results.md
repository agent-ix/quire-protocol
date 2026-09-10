---
id: TC-008
title: "Reproduce results under equivalent presentations"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/NFR-001
    type: verifies
---
# TC-008: Reproduce results under equivalent presentations

## Description

Verify byte and semantic stability across equivalent input presentations and
discrimination across semantic mutations.

## Test Procedure

Generate map-member-order, locale, allocation-layout, and permitted causally
independent arrival-order permutations while preserving all admitted identities,
authored sequences, and causal order. Then independently mutate the source,
candidate/profile, model, compiled-protocol, request, binding, input,
observation, each decision-scope and surrounding-execution progress/closure
axis, complete-global-conformance closure, producer, implementation, and backend
identity.

## Expected Results

Equivalent presentations produce byte-identical RFC 8785 artifacts, identical
ordered semantic outcomes, and complete identical provenance. Every semantic
mutation changes the affected identity or returns a typed refusal. Unordered
semantic sets use their declared canonical order; authored sequences and causal
order are never presentation-sorted.
