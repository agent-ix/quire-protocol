---
id: TC-054
title: "Assess loop bound and report proof as undischarged"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-056
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-059
    type: verifies
---
# TC-054: Assess loop bound and report proof as undischarged

## Description

Verify assess loop bound and report proof as undischarged over a checked package and an admitted observation history, with no exploration of the protocol.

## Test Procedure

Assess loops at the maximum minus one, the maximum and one over, a loop with no maximum, a loop inside a `repeat` under two outer ordinals, an undeclared loop, workflow identities QSL's key cannot take, a history ending inside a loop, and a loop with an invariant and a lexicographic variant, then request their discharge. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

The bound is judged exactly; an unfinished loop is not exited; declarations stay undischarged; the discharge request is unsupported; no proof status is produced.
