---
id: TC-052
title: "Assess queue capacity, overflow outcome and order"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-054
    type: verifies
---
# TC-052: Assess queue capacity, overflow outcome and order

## Description

Verify assess queue capacity, overflow outcome and order over a checked package and an admitted observation history, with no exploration of the protocol.

## Test Procedure

Assess histories within and over a literal capacity, one per overflow policy matching and mismatching, key-ordered and key-interleaved delivery, a channel whose premise forbids loss, `unordered` channels with a head, a tail and a new loss selector, a loss whose dropped message disagrees with the queue, and a symbolic-capacity channel. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Capacity, overflow and order violations name the channel and the authored and observed values; a symbolic capacity returns the unsupported disposition.
