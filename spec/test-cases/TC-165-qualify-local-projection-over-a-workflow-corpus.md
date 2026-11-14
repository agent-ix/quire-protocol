---
id: TC-165
title: "Qualify local-projection over a workflow corpus on QSL's linked engine"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-150
    type: verifies
---
# TC-165: Qualify local-projection over a workflow corpus on QSL's linked engine

## Description

Verify, with no stand-in, that projecting each role of each workflow in a corpus settles the truth QSL's engine decides, and that a global-conformance result is unchanged beside a projection claim.

## Test Procedure

Compile a three-role protocol in variants whose choice branches the warehouse can tell apart, cannot tell apart, and merges. Register QSL's engine and dispatch a `local-projection` claim onto each role. Then dispatch one projection claim and one global-conformance result together. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Every role satisfies where the branches are distinguishable or merge; only the warehouse is refuted where they are not. The global-conformance result equals the one returned without the projection claim.
