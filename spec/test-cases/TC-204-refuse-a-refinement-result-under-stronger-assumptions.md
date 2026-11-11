---
id: TC-204
title: "Refuse a refinement result whose assumptions are stronger than the request's"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-204
    type: verifies
---
# TC-204: Refuse a refinement result whose assumptions are stronger than the request's

## Description

Verify identity inclusion of a result's retained assumptions in the request's and the carrying of a backend-reported strengthened-assumption finding, with a stand-in for the QSL call boundary and no local analysis.

## Test Procedure

Request a refinement claim with assumptions. Feed stand-in results that retain the same set, one extra identity on a proof, one extra identity on a violation, a strict subset, an empty set, an omitted field, and the backend's own `protocol-assumption-not-established` finding, beside conformance and composition claims. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Equal and subset sets are carried; an omitted field is refused naming it; an extra identity refuses the disposition with `identity-mismatch` naming it and no success value; independent dispositions are unchanged; the backend's own finding is carried unchanged.
