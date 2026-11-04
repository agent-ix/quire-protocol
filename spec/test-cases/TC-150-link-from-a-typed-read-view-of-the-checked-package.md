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

Verify the link layer over the checked package's nodes and QSL's causal relation.

## Test Procedure

Link **R**'s protocol and compare its lists and edge set with `protocol_view` and `causal_relation`; inspect the source for any edge construction; edit views for a zero-progress iteration, duplicate role authorities and conflicting channel premises; link an absent id, a `protocol_reference` carrier and a non-protocol node; compile-fail the retired types; read a window binding; re-run FR-002-AC-1 to AC-5 over **R**. Tag the tests `#[trace("TC-150", "FR-044-AC-n")]`.

## Expected Results

Lists and edges equal the view's; no local edge derivation; each consumer refusal located; typed refusals with no partial representation; retired types unresolved; the window binding on its retained spelling; FR-002's criteria hold over **R**.
