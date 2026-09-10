---
id: TC-002
title: "Link and discriminate finite protocol structure"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-002
    type: verifies
---
# TC-002: Link and discriminate finite protocol structure

## Description

Verify exact typed identity, causal control, joins, bounded iteration, channel
premises, and choice visibility across the selected finite representation.

## Test Procedure

Generate valid split-shipment protocols and every permitted branch interleaving.
Independently substitute role/instance/message/channel/delivery/attempt/effect
identities; add cycles, invalid joins, missing iteration limits, conflicting
channel rules, and owner-invisible choice facts; permute equal-time input.

## Expected Results

Valid structures link to identical semantic graphs. Each invalid mutation returns
the exact located refusal, and presentation permutations add no causal edge.

