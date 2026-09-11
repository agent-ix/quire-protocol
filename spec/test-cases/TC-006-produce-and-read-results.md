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
surrounding-execution progress, surrounding-execution closure, and
complete-global-conformance closure; attempt to use any one as another. For
each global-closure state, independently omit, duplicate, cross-wire, change
scope on, and change identity on every required execution, branch, and workflow
premise. Exercise `not-required` on each non-global claim and refuse it on a
global claim or when it carries a global premise. Keep one independently
settled per-obligation truth fixed through every global-closure mutation.

## Expected Results

All valid rows round-trip to byte-identical canonical results. Every scope-axis
collapse, cross-wire, forbidden combination, or lineage defect refuses.
Complete global success requires the exact `closed` global record and complete
premise set; an incomplete record prevents global success without changing the
fixed per-obligation truth. A late contradiction creates a new linked result
while the earlier bytes and identity remain unchanged.
