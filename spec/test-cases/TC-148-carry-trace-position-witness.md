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
or `decisive-counterexample` carries a byte-exact trace-position witness, with
each component present exactly as `ix://agent-ix/quire-specification/FR-351`
states for the protocol family, under contract version
`quire.protocol.result/v1-draft.2`, that every other
settlement basis carries none, and that an absent `native-run-result/2` record
settles `unsupported` rather than a substitute witness.

## Test Procedure

Evaluate a realizability and a refinement claim each settled
`decisive-witness`, `decisive-counterexample`, `closed-scope`, `unsettled` and
`unavailable`. For each decisive case, remove, reorder or normalize one of the
witness's components (deciding element identity, index, value path, trace
position) in turn, and separately substitute a default for a component the
admitted record leaves absent. Separately evaluate a decisive settlement whose
analysis backend produces no `native-run-result/2` record.

## Expected Results

Every decisive settlement carries exactly one witness whose components
byte-match the admitted separating-witness record, with a component the record
leaves absent carried as absence; every non-decisive settlement carries none.
Removing, reordering or normalizing any witness component, or substituting a
default for an absent one, refuses the result rather than repairing it. The
absent-record case settles `unsupported` with a warning naming the required
capability and carries no substitute witness. Every emitted result carries the
explicitly selected contract version `quire.protocol.result/v1-draft.2`.
