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

Verify that, with no candidate serving a kind, the claim settles `unsupported` with QSL's cause, truth and basis `unavailable`, a warning and no error, while global conformance is preserved, and that registering a serving backend changes the result of a byte-identical request.

## Test Procedure

Request `local-projection`, `realizability` and `global-conformance` over a satisfiable conformant trace with an empty registry (also naming a backend in a second run), then with a stand-in serving only `refinement`, then re-run the byte-identical request after a stand-in serving `local-projection` is registered. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Unserved claims settle `unsupported` with cause `unsupported_projection`/`unsupported-requested-capability`, truth and basis `unavailable`, a warning naming the kind (and the named backend), and no error, hold or refusal; none reports as holding. The global-conformance result is unchanged. Retained fragment, backend, assumptions, premises and source identities equal the request's and no result, witness or projected-artifact field is present. The refinement-only stand-in records no call. After registration the `local-projection` disposition equals its result and `realizability` stays `unsupported`.
