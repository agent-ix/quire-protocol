---
id: TC-050
title: "Link role, queue, join and loop declarations"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-050
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-051
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-052
    type: verifies
---
# TC-050: Link role, queue, join and loop declarations

## Description

Verify link role, queue, join and loop declarations over an admitted checked package, with no observation history and no exploration of the protocol.

## Test Procedure

Link checked packages declaring roles of each lifetime and bound, channels of each capacity, ordering and overflow policy, and joins and loops of each policy, and compare each linked field with the checked package's value. Link twice and compare. Confirm no identity is minted and no symbolic value is evaluated. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Each linked field equals the checked package's; absent bounds and policies stay absent; symbolic capacity and predicate stay unevaluated; repeated linking is equal.
