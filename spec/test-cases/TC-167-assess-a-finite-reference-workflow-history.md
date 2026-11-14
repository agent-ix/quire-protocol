---
id: TC-167
title: "Assess a finite reference workflow history on every dynamic axis"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-050
    type: verifies
---
# TC-167: Assess a finite reference workflow history on every dynamic axis

## Description

Verify that one finite observed history of a choreography (a replicated worker role, a bounded queue, a fan-out and join, a bounded repeat) is assessed on the role, channel, join and loop axes, that each axis is judged separately, and that assessing the same history twice gives the same result.

## Test Procedure

Compile the dynamic-roles choreography with QSL's writer, admit and link it, and assess a conforming run and a faulty run (worker population over its bound, queue over its capacity, a join settled before its threshold, a loop over its maximum), each twice.

## Expected Results

The conforming run holds on every axis. The faulty run fails on each axis and on no other; both results are identical on the second pass.
