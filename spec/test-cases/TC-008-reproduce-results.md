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
accepted source-baseline/profile, model, compiled-protocol, request, binding, input,
observation, each decision-scope and surrounding-execution progress/closure
axis, complete-global-conformance closure, producer, implementation, and backend
identity.

## Expected Results

Equivalent presentations produce byte-identical RFC 8785 artifacts, identical
ordered semantic outcomes, and complete identical provenance. Every semantic
mutation changes the affected identity or returns a typed refusal. Unordered
semantic sets use their declared canonical order; authored sequences and causal
order are never presentation-sorted.

## Traceability
This test case additionally backs criteria on the object requirements FR-016, FR-017, which cite it as their verification. Those criteria are exercised by the same procedure above; a change here that stops exercising them is a change to their coverage, not only to this test case's.

This test case additionally backs `NFR-008-AC-7` of
[NFR-008](../non-functional/NFR-008-carry-every-refusal-in-its-return-value.md),
which names it as its verification: two runs on identical inputs return
non-success values comparing equal component by component, with the observed
elapsed time of a resource-incomplete outcome excluded. The reproduction method
is NFR-001's and is exercised by the procedure above; NFR-008 restates none of
it.
