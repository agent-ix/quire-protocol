---
id: TC-009
title: "Enforce every protocol resource boundary"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/NFR-002
    type: verifies
---
# TC-009: Enforce every protocol resource boundary

## Description

Verify each caller-supplied resource boundary and discriminate every guard.

## Test Procedure

For each input-byte, graph, branching, iteration, observation, lineage, output,
operation, memory, and elapsed-time dimension, execute cases below, at, and above
one positive owner-supplied limit. Remove each guard in an isolated mutation.

## Expected Results

Below-limit work may proceed, at-limit behavior matches the exact inclusive or
exclusive contract, and over-limit work refuses before exceeding the bound. Each
guard-removal mutant is killed by its assigned test for the intended reason.

