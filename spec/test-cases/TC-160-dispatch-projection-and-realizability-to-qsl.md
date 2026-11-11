---
id: TC-160
title: "Dispatch local-projection and realizability claims to QSL's backend"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-150
    type: verifies
---
# TC-160: Dispatch local-projection and realizability claims to QSL's backend

## Description

Verify that each claim kind reaches only a registered candidate serving it under FR-290, that the backend's result, refusal, `unsupported` cause or failure is carried unchanged, and that the library computes no verdict of its own.

## Test Procedure

Dispatch `local-projection`, `realizability` and `global-conformance` together against stand-in backends installed as registrations in QSL's registry snapshot at the FR-331 provider boundary (never as caller callbacks, and never as entries of FR-005's closed table). Each stand-in records every call. Use stand-ins serving both kinds, serving only each kind, returning one fixed sentinel result for every request, returning no verdict, returning a refusal, returning an `unsupported` cause, returning `unsupported_projection`/`tool-unavailable`, and failing. Register two stand-ins serving `realizability` and request it naming neither, naming one, and naming an unregistered backend, under every registration order. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Each claim reaches exactly one candidate that serves its kind, once, with its fragment, assumptions and source identities. Dispositions equal the sentinel results byte for byte in the backend's vocabulary, whatever the fragment; no verdict appears where the backend returned none; a failing backend yields `failed` for its claim alone with no second backend called; the global-conformance disposition equals its stand-alone value. Two unnamed candidates settle `ambiguous-backend` with no call, an unregistered name settles `unknown-backend`, and a named candidate receives the claim, identically under every registration order.
