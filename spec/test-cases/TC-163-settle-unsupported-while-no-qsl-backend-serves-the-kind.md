---
id: TC-163
title: "Settle projection and realizability claims unsupported while no backend serves the kind"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-153
    type: verifies
---
# TC-163: Settle projection and realizability claims unsupported while no backend serves the kind

## Description

Verify that, with no backend serving a kind, the claim settles `unsupported` with QSL's cause name, truth and basis `unavailable`, a warning, and no error, while global conformance is preserved, and that adding a serving backend changes the result without changing the dispatch path.

## Test Procedure

Request `local-projection`, `realizability` and `global-conformance` over a satisfiable conformant trace with an empty backend set, then with a stand-in serving only `refinement`, then with a stand-in serving `local-projection` added. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Unserved claims settle `unsupported` with cause name `unsupported-requested-capability`, truth and basis `unavailable`, a warning naming the kind, and no error, hold or refusal; none reports as holding. The global-conformance result is unchanged. The refinement-only stand-in records no call. With the added stand-in, `local-projection` equals its result through the same dispatch path and `realizability` stays `unsupported`.
