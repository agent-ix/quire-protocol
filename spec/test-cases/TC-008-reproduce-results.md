---
id: TC-008
title: "Reproduce results under equivalent presentations"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/NFR-001
    type: verifies
---
# TC-008: Reproduce results under equivalent presentations

## Description

Verify byte and semantic stability across equivalent input presentations and
discrimination across semantic mutations.

## Test Procedure

Generate map-order, source-format, locale, independent-arrival-order, and
allocation-layout permutations for the same semantic input, then independently
mutate every identity-bearing semantic input and dependency.

## Expected Results

Equivalent inputs produce byte-identical RFC 8785 artifacts and outcomes. Every
semantic mutation changes the affected identity or returns a typed refusal.

