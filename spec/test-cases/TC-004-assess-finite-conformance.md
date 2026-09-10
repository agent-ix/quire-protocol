---
id: TC-004
title: "Assess finite global conformance"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-004
    type: verifies
---
# TC-004: Assess finite global conformance

## Description

Verify finite conformance across valid interleavings and every distinct adverse
observation, decision-scope progress/closure, surrounding-execution
progress/closure, complete-global-conformance closure, completeness, execution,
and budget state.

## Test Procedure

Replay the complete accepted branching/compensation corpus in batch and
incrementally. Permute causally independent events, then inject unexpected,
missing, misordered, duplicate-effect, cross-workflow, open, incomplete,
unsupported, failed, and exhausted cases one at a time. Close decision scope,
surrounding execution, and complete-global-conformance scope independently, then
cross-wire each asserted closure identity and progress point.

## Expected Results

Every valid permutation agrees. Each adverse case names its exact obligation and
state. Closing one scope does not close either other scope. No open, incomplete,
unsupported, failed, or exhausted case reports complete global conformance.
