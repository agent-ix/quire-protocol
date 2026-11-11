---
id: TC-055
title: "Map QSL Unsupported and requires-bound outcomes to dispositions"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-057
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-058
    type: verifies
---
# TC-055: Map QSL Unsupported and requires-bound outcomes to dispositions

## Description

Verify map QSL Unsupported and requires-bound outcomes to dispositions over a checked package and an admitted observation history, with no exploration of the protocol.

## Test Procedure

Feed each QSL Unsupported cause ADR-010 lists, an unknown cause, and requires-bound outcomes for a role, a channel and a loop, alongside independently supported claims. Inspect the arguments of the QSL calls made. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Each disposition is typed non-success with QSL's name unchanged and the construct's node id; independent dispositions and axes are unchanged; no bound is chosen and none is passed.
