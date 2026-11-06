---
id: TC-150
title: "Link from a typed read view of the checked package"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-044
    type: verifies
---
# TC-150: Link from a typed read view of the checked package

## Description

Verify the link layer over the checked package's nodes and QSL's causal relation,
including the consumer-owned residual checks added after stage 2.

## Test Procedure

Link **R**'s protocol and compare its lists and edge set with `protocol_view` and
`causal_relation`; inspect the source for any edge construction; edit checked
nodes for a zero-maximum repeat, duplicate role authorities and conflicting
channel premises; verify each consumer refusal names its node ids and available
source-map loci; link an absent id, a `protocol_reference` carrier and a
non-protocol node; compile-fail the retired types; verify linking succeeds
without any window-binding package member and the linker imports no window view
from QSL; re-run FR-002-AC-1 to AC-5 over **R**, including concurrent identity,
legal interleavings, equal-time input reordering and changed temporal/clock
selection. Tag each test with its exact acceptance criterion using
`#[trace("FR-044-AC-n", "TC-150")]`.

## Expected Results

Lists and edges equal the view's; no local edge derivation; each consumer
refusal is typed, located when an owner locus exists, and returns no partial
representation; owner refusals remain unchanged; retired types unresolved; no
window package member or QSL window view is required; FR-002's criteria hold
over **R**.
