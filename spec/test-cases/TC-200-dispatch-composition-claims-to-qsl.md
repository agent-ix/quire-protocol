---
id: TC-200
title: "Dispatch composition claims to QSL's composition backend"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-200
    type: verifies
---
# TC-200: Dispatch composition claims to QSL's composition backend

## Description

Verify dispatch of a composition claim to a recording stand-in for the QSL call boundary, the unsupported path with no accepted backend, independence of other claims and the absence of any local discharge, with a stand-in for the QSL call boundary and no local analysis.

## Test Procedure

Request a composition claim whose premises are all supplied by matching guarantees, once with a stand-in backend returning a sentinel result and once with no accepted FR-292 backend, beside finite global conformance and a refinement claim. Inspect the arguments of the calls made to the stand-in. Remove or substitute each retained disposition component one at a time. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

One composition request item reaches the stand-in and its sentinel is carried; with no backend the claim is `unsupported` with a warning naming the capability although every premise is supplied; other dispositions are unchanged; no global-conformance premise set appears as evidence.
