---
id: TC-166
title: "Dispatch realizability claims to QSL's protocol analysis engine"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-150
    type: verifies
---
# TC-166: Dispatch realizability claims to QSL's protocol analysis engine

## Description

Verify that a `realizability` claim is decided by QSL's real `protocol-analysis` engine registered at the provider boundary, with no stand-in, and that the engine's result and separating-witness record are carried unchanged.

## Test Procedure

Build a three-role protocol and a one-object model with QSL's public front end (`qsl_cst::parse`, `qsl_forms::build_unit`, `admit_unit`, `PackageDeclarations::assemble`, `CheckedPackage::link`), emit and admit it through Contract IR's reader, and register QSL's engine under QSL's descriptor. Dispatch a `realizability` claim for a protocol whose roles can tell every branch apart and for one with a hidden choice. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

The realizable protocol settles `satisfied` on `closed-scope`; the hidden choice settles `violated` on `decisive-counterexample` and carries QSL's separating-witness record unchanged. The selected backend is `protocol-analysis`.
