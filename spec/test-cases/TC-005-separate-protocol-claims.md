---
id: TC-005
title: "Separate protocol claim dispositions"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-005
    type: verifies
---
# TC-005: Separate protocol claim dispositions

## Description

Verify that conformance, monitorability, projection, realizability, refinement,
and composition remain independently requested and dispositioned.

## Test Procedure

Evaluate monitorable-but-unprojectable, satisfiable-but-unrealizable,
circular-assumption, unsupported-backend, and failed-backend controls while a
valid finite conformance result is also present. For every returned disposition,
remove or substitute its admitted fragment, backend, assumption, communication,
visibility, environment, source, and derived-artifact identity one at a time.

## Expected Results

No weaker fact establishes a stronger claim. Unsupported and failed claims remain
explicit, invoke no fallback, and do not erase the independent conformance result.
Every exact provenance tuple round-trips; each missing or substituted member
refuses only its affected disposition and cannot alter another claim.
