---
id: TC-148
title: "Carry a trace-position witness on the protocol result wire"
type: TC
relationships:
  - target: ix://agent-ix/quire-protocol/FR-302
    type: verifies
---
# TC-148: Carry a trace-position witness on the protocol result wire

## Description

Verify that a `realizability` or `refinement` claim settled `decisive-witness`
or `decisive-counterexample` carries a byte-exact trace-position witness under
contract version `quire.protocol.result/v1-draft.2`, that every other
settlement basis carries none, and that an absent `native-run-result/2` record
settles `unsupported` rather than a substitute witness.

## Test Procedure

Evaluate a realizability and a refinement claim each settled
`decisive-witness`, `decisive-counterexample`, `closed-scope`, `unsettled` and
`unavailable`. For each decisive case, remove, reorder or normalize one of the
witness's four components (deciding element identity, index, value path, trace
position) in turn. Separately evaluate a decisive settlement whose analysis
backend produces no `native-run-result/2` record.

## Expected Results

Every decisive settlement carries exactly one witness whose four components
byte-match the admitted `native-run-result/2` record; every non-decisive
settlement carries none. Removing, reordering or normalizing any witness
component refuses the result rather than repairing it. The absent-record case
settles `unsupported` with a warning naming the required capability and
carries no substitute witness. Every emitted result carries contract version
`quire.protocol.result/v1-draft.2`, and no `v1-draft.1` artifact is accepted in
its place.
