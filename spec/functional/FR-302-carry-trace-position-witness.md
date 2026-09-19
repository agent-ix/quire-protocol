---
id: FR-302
title: "Carry a trace-position witness on the protocol result wire"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/US-002
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-006
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-031
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-032
    type: depends_on
  - target: ix://agent-ix/quire-protocol/AD-001
    type: references
  - target: ix://agent-ix/quire-specification/FR-290
    type: references
---
# FR-302: Carry a trace-position witness on the protocol result wire

## Description

When a `realizability` or `refinement` claim settles with basis
`decisive-witness` or `decisive-counterexample`, the result producer SHALL
carry one trace-position witness record — deciding element identity, index,
value path and trace position — imported byte-exactly from the accepted
family-neutral separating-witness record `native-run-result/2`, minted by
`agent-ix/quire-specification#114`, under the next `quire.protocol.result`
contract version this repository publishes as `v1-draft.2`.

## Inputs

One settled `realizability` or `refinement` claim whose settlement basis is
`decisive-witness` or `decisive-counterexample`, and the admitted
`native-run-result/2` separating-witness record for that settlement.

## Outputs

One canonical result whose settlement carries exactly one trace-position
witness (deciding element identity, index, value path, trace position) when
the basis is decisive, and none when it is not.

## Behavior

When settlement basis is `decisive-witness` or `decisive-counterexample` for a
`realizability` or `refinement` claim, the result producer SHALL carry the
imported deciding-element identity, index, value path and trace position from
the admitted `native-run-result/2` record.

A settlement whose basis is `closed-scope`, `unsettled` or `unavailable`
carries no witness. A decisive basis without its witness refuses the result.

The four witness components are copied byte-exactly from the admitted
`native-run-result/2` record; normalizing, recomputing, reordering or
supplying a display index refuses the result rather than repairing it.

A settlement whose analysis backend cannot produce a `native-run-result/2`
record settles `unsupported` with a warning naming the required capability.

The carrying contract version is `quire.protocol.result/v1-draft.2`; the prior
`v1-draft.1` identifier is superseded for the witness-bearing settlement bases
and is not read as `v1-draft.2` by any strict reader. This requirement mints no
version label of its own and consumes exactly what
`ix://agent-ix/quire-specification/FR-240` next publishes for the wire's
`contractVersion`; this repository selects from that published identifier
rather than authoring one locally.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-302-AC-1 | Every `realizability` or `refinement` claim settled `decisive-witness` or `decisive-counterexample` carries exactly one trace-position witness whose deciding-element identity, index, value path and trace position byte-match the admitted `native-run-result/2` record. | Test (TC-148) |
| FR-302-AC-2 | A settlement basis outside `decisive-witness`/`decisive-counterexample` carries no witness, and a decisive basis missing its witness refuses the result. | Test (TC-148) |
| FR-302-AC-3 | The witness's four components are copied byte-exactly from the admitted record; normalizing, recomputing, reordering or supplying a display index refuses the result. | Test (TC-148) |
| FR-302-AC-4 | A settlement whose analysis backend cannot produce a `native-run-result/2` record settles `unsupported` with a warning naming the required capability. | Test (TC-148) |
| FR-302-AC-5 | The witness-bearing result carries contract version `quire.protocol.result/v1-draft.2`; a `v1-draft.1` artifact is never accepted as a `v1-draft.2` result and the two versions are never mixed in one strict read. | Test (TC-148) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Separating-witness record | `native-run-result/2` (`agent-ix/quire-specification#114`) | authored, keyed (deciding element identity, index, value path, trace position) |
| Settlement basis | `ix://agent-ix/quire-specification/FR-243` | value |
| Contract version | `ix://agent-ix/quire-specification/FR-240` | value |

### Accepted owner rulings

`agent-ix/quire-specification#114` mints `native-run-result/2` as a
family-neutral separating-witness record shared by codegen, quire-verification
and quire-protocol consumers; exhausted and cancelled settlements carry no
witness, and an `unsupported` settlement emits no substitute witness. This
requirement is the quire-protocol ticket that record names as the consumer
whose result wire bumps its draft revision; it defines this repository's
carrier for that record and mints no shape of its own.

## Dependencies

- [FR-006](./FR-006-produce-canonical-protocol-results.md),
  [FR-031](./FR-031-canonical-result-identity-and-digest.md) and
  [FR-032](./FR-032-retain-orthogonal-result-axes.md) govern the
  `v1-draft.1` canonical result this requirement extends.
- `ix://agent-ix/quire-specification/FR-240` publishes the `contractVersion`
  member this requirement consumes.
- `ix://agent-ix/quire-specification/FR-290` names `realizability` and
  `refinement` as the claim kinds whose decisive settlements carry a witness.
- `agent-ix/quire-specification#114` mints the `native-run-result/2` record
  this requirement carries.
