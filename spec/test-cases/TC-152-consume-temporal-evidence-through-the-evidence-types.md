---
id: TC-152
title: "Consume temporal evidence through the in-memory evidence types"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-046
    type: verifies
---
# TC-152: Consume temporal evidence through the in-memory evidence types

## Description

Verify the assessment over `EvidencePackage` and the removal of the producer contracts.

## Test Procedure

Assess **R**'s four temporal clauses on both package routes; read the producer-contract registry; scan the source for the retired types; trigger each cause of QSL FR-516's cause table and each `PackageMismatch` variant; run the IT-002 scenarios on the evidence route with an unavailable and a withdrawn anchor. Tag the tests `#[trace("TC-152", "FR-046-AC-n")]`.

## Expected Results

Equal records on both routes; no producer contract string; no retired type; each cause carried by name with no charge; each mismatch refused before a call; unavailable anchors never read as Boolean.
