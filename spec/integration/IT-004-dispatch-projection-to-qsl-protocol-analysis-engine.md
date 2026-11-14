---
id: IT-004
title: "Dispatch projection claims to QSL's protocol analysis engine"
type: IT
relationships:
  - target: ix://agent-ix/quire-protocol/FR-150
    type: verifies
---
# IT-004: Dispatch projection claims to QSL's protocol analysis engine

## Objective

Verify that a protocol compiled by QSL's writer, admitted and linked by this
library, has its `local-projection` claims decided by QSL's own
`protocol-analysis` engine with no stand-in, and that the engine's result is
carried unchanged beside an unrelated global-conformance result.

## Target Integration

QSL compiles and decides; this library admits, links, binds premises and
carries. This library computes no projection and no witness.

| Owner | Supplies | This repository must not |
| --- | --- | --- |
| QSL | The checked package, the engine, its descriptor and its result | Be asked to project or realize locally |
| This repository | Admission, linking, premise binding, dispatch, carriage | Substitute a stand-in or alter the engine's record |

## Preconditions

QSL's linked engines are available in process. `composition` and `refinement`
have no linked QSL engine, and `realizability` needs a QSL protocol subject the
checked handoff does not carry (TL-391); this procedure covers
`local-projection` only.

## Inputs

A three-role protocol (`Buyer`, `Seller`, `Warehouse`) whose `Seller` owns a
choice, in four variants: branches the warehouse tells apart, branches it
cannot tell apart, and branches that merge.

## Test Procedure

1. Compile each variant, admit and link it.
2. Register QSL's engine under QSL's descriptor.
3. Dispatch a `local-projection` claim onto each role.
4. Dispatch a request holding one projection claim and one global-conformance
   result.

## Expected Results

Each role settles the truth the engine decides; the warehouse alone is refuted
where its branches cannot be told apart; the global result is returned
unchanged.

## Traceability

Verified by TC-164 and TC-165.
