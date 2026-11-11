---
id: TC-053
title: "Assess join settlement per policy"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-055
    type: verifies
---
# TC-053: Assess join settlement per policy

## Description

Verify assess join settlement per policy over a checked package and an admitted observation history, with no exploration of the protocol.

## Test Procedure

Assess `all`, `any` and `quorum(n)` joins at n-1, n and n+1 completed branches with a duplicated branch observation, outstanding `continue` and `cancel` branches, a predicate join, an `all` join with no outstanding rule and `any` and `quorum(n)` joins with none. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Settlement is judged per policy and threshold with branches counted once; predicate and unruled joins return the unsupported disposition.
