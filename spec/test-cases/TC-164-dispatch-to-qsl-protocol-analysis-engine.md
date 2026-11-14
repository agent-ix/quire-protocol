---
id: TC-164
title: "Dispatch local-projection claims to QSL's protocol analysis engine"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-150
    type: verifies
  - target: ix://agent-ix/quire-protocol/FR-153
    type: verifies
---
# TC-164: Dispatch local-projection claims to QSL's protocol analysis engine

## Description

Verify that a `local-projection` claim is decided by QSL's real `protocol-analysis` engine registered at the provider boundary, with no stand-in, and that the engine's result and separating-witness record are carried unchanged.

## Test Procedure

Compile a three-role protocol with QSL's writer, admit it, register QSL's engine under QSL's descriptor, and dispatch a `local-projection` claim onto one role for a protocol whose choice branches the role can tell apart, for one it cannot, and with an unknown profile the engine refuses. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

The distinguishable protocol settles `satisfied`; the indistinguishable one settles `violated` and carries QSL's separating-witness record unchanged; the refused request is not a result and holds neither `satisfied` nor `violated`. The selected backend is `protocol-analysis`.
