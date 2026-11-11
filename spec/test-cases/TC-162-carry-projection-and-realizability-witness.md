---
id: TC-162
title: "Carry a projection or realizability witness joined to the linked graph"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-152
    type: verifies
---
# TC-162: Carry a projection or realizability witness joined to the linked graph

## Description

Verify that a backend witness, counterexample or projected local artifact is carried unchanged beside the source identities its disposition retains under FR-030, and that a disposition retaining no source identity is refused alone.

## Test Procedure

Dispatch `realizability` claims against a stand-in returning a `decisive-witness` result, a `decisive-counterexample` result and a deliberately non-minimal witness, and a `local-projection` claim against a stand-in returning a projected artifact. Then use a decisive request whose disposition retains no source identity, and an artifact whose identity disagrees with its derived tuple (FR-152-AC-6). Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Each decisive result carries one trace-position witness per FR-302 byte for byte, beside the claim's retained source identities. The non-minimal witness is carried byte for byte. The disposition retaining no source identity and the disagreeing artifact each refuse only their own disposition.
