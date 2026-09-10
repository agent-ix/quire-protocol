---
id: TC-006
title: "Produce and strictly read canonical protocol results"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-006
    type: verifies
---
# TC-006: Produce and strictly read canonical protocol results

## Description

Verify every canonical result combination, identity, support relation, decisive
open-subject settlement, and immutable supersession rule.

## Test Procedure

Generate every valid FR-061 table row and mutate each orthogonal state and
identity independently. Exercise independent/shared/derived/unknown support,
missing predecessors, cycles, self-reference, decisive and non-decisive open
prefixes, missing support, and later contradictions. Independently mutate and
cross-wire decision-scope progress, decision-scope closure,
surrounding-execution progress, and surrounding-execution closure; attempt to
use any one of them as another or as complete-global-conformance closure.

## Expected Results

All valid rows round-trip to byte-identical canonical results. Every scope-axis
collapse, cross-wire, forbidden combination, or lineage defect refuses. A late
contradiction creates a new linked result while the earlier bytes and identity
remain unchanged.
