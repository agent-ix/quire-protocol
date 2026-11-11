---
id: TC-147
title: "Admit protocol refinement dispatch"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-301
    type: verifies
---
# TC-147: Admit protocol refinement dispatch

## Description

Verify that a requested refinement claim is dispatched to QSL's refinement
backend and its result carried once FR-176 and FR-177 are admitted, that it never
borrows the complete-global-conformance premise set as its own evidence, that
its absence path preserves the independent global conformance result, and that
an absent analysis backend settles `unsupported` rather than a hold, blocker or
refusal error.

## Test Procedure

Evaluate a monitorable protocol with a requested refinement claim, using a
test QSL refinement backend that returns known sentinel results, under three
conditions: (1) FR-176/FR-177 admitted with a satisfying implementation/
specification pair, (2) FR-176/FR-177 admitted with a violating pair, and (3)
no accepted FR-292 refinement backend. For each, also request finite global
conformance in the same dispatch. In case (1) the backend returns a
simulation certificate, in case (2) a counterexample and in a fourth run a
refusal with a distinct cause; the library holds no local refinement
algorithm to compute a substitute. Remove or substitute the refinement
disposition's admitted fragment, backend, assumption identities and source
identity one at a time.

## Expected Results

Cases (1) and (2) settle refinement independently of the global-conformance
premise set; neither disposition erases the other. Case (3) returns an
explicit `unsupported` refinement disposition with a warning naming the
required capability, and the finite global conformance result remains present
and unchanged. Every removed or substituted component refuses only the
refinement disposition without altering the global-conformance result.
The certificate, counterexample and refusal cause in the disposition equal the
sentinel values the backend returned, byte for byte, in the backend's own
vocabulary (FR-301-AC-6). A backend that returns a verdict different from the
one the paired implementation and specification would yield under a local
construction still has its returned verdict carried: the disposition never
differs from the backend's result, and no verdict appears when the backend
returns none.
