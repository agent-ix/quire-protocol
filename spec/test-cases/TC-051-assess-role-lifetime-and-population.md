---
id: TC-051
title: "Assess role lifetime and population bound"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-053
    type: verifies
---
# TC-051: Assess role lifetime and population bound

## Description

Verify assess role lifetime and population bound over a checked package and an admitted observation history, with no exploration of the protocol.

## Test Procedure

Assess histories inside, before and after a lifetime, after the end of a workflow, scope and until(p) lifetime, a scope role's compensation attempt with an open registration, an until(p) instance acting after p holds but before its retire, at the population bound and one over it, a static role's activity and a lifecycle fact for it, a fact naming an undeclared role, exchanged positions of concurrent observations read from the linked causal relation, and two workflow instances whose live instances together exceed the bound. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Violations name the instance or role, bound and count; a static role's bound is reported absent and its activity is never a violation; population and workflow end are judged per workflow instance; results are invariant under exchanges of concurrent observations.
