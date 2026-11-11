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

Assess histories inside, before and after a lifetime, at the population bound and one over it, with an absent bound, and under permuted presentation orders. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Violations name the instance or role, bound and count; an absent bound is reported absent; results are order invariant.
