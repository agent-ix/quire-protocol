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

Verify that a backend witness, counterexample or projected local artifact is carried unchanged with its source identities joined to linked-graph nodes, and that an unresolvable identity refuses only its disposition.

## Test Procedure

Dispatch `realizability` claims against a stand-in returning a `decisive-witness` result, a `decisive-counterexample` result and a deliberately non-minimal witness, and a `local-projection` claim against a stand-in returning a projected artifact. Then return a source identity absent from the linked graph, and an artifact whose identity disagrees with its derived tuple. Tag each test with the exact acceptance criterion it verifies.

## Expected Results

Each decisive result carries one trace-position witness per FR-302 whose source identities each equal one linked-graph node identity. The non-minimal witness is carried byte for byte. The unresolvable identity and the disagreeing artifact each refuse only their own disposition.
