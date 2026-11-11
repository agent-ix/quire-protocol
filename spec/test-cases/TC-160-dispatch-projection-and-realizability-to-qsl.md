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

Verify that each claim kind is sent only to a backend serving it, that the backend's result, refusal, `unsupported` cause or failure is carried unchanged, and that the library computes no verdict of its own.

## Test Procedure

Dispatch `local-projection`, `realizability` and `global-conformance` together against stand-in backends that record every call and return sentinel results: one serving both kinds, one serving only each kind, one returning a verdict opposite to a construction over the linked graph, one returning no verdict, one returning a refusal, one returning an `unsupported` cause, and one failing. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Each claim reaches exactly one backend that serves its kind, once, with its fragment, assumptions and source identities. Dispositions equal the sentinel results byte for byte in the backend's vocabulary; no verdict appears where the backend returned none; a failing backend yields `failed` for its claim alone with no second backend called; the global-conformance disposition equals its stand-alone value.
