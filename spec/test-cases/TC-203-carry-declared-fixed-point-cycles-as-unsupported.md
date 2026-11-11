---
id: TC-203
title: "Carry declared fixed-point cycles as typed unsupported"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-203
    type: verifies
---
# TC-203: Carry declared fixed-point cycles as typed unsupported

## Description

Verify the typed unsupported disposition for a declared cycle and the absence of any local fixed point, with a stand-in for the QSL call boundary and no local analysis.

## Test Procedure

Request a composition whose declaration holds a two-guarantee cycle. Feed a stand-in returning `unsupported-requested-capability`, `premise-unproved`, a `malformed-declaration` refusal, and a proved guarantee for one cycle member. Request an acyclic composition beside it. Inspect the calls made. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

The cycle settles `unsupported` with QSL's cause name and node ids; other outcomes are carried unmapped; no cycle guarantee is an assumption; one call is made; the acyclic claim is unchanged.
