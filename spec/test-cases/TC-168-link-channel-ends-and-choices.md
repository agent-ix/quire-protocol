---
id: TC-168
title: "Link channel ends and choices from QSL's read view"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-051
    type: verifies
---
# TC-168: Link channel ends and choices from QSL's read view

## Description

Verify that a linked channel carries its sender, receiver and message type and a linked choice its owner and visible set exactly as QSL's read view publishes them, and that the dispatcher forwards the channel ends it linked.

## Test Procedure

Compile a three-role protocol with a choice and a fan-in/fan-out of channels of distinct message types with QSL's writer, admit and link it, compare each linked channel and choice with QSL's `channel_endpoints` and `choice_visibility`, and dispatch a claim to a stand-in that records the call.

## Expected Results

Every linked channel equals QSL's reading; two channels of different message types carry different type nodes; the choice carries its owner and visible nodes in order; the recorded call carries the linked channel ends.
