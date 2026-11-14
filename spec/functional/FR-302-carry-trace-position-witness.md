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
  - target: ix://agent-ix/quire-specification/FR-240
    type: references
  - target: ix://agent-ix/quire-specification/FR-243
    type: references
  - target: ix://agent-ix/quire-specification/FR-290
    type: references
  - target: ix://agent-ix/quire-specification/FR-351
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-352
    type: depends_on
---
# FR-302: Carry a trace-position witness on the protocol result wire

## Description

When a `realizability` or `refinement` claim settles with basis
`decisive-witness` or `decisive-counterexample`, the result producer SHALL
carry one trace-position witness record — deciding element identity, index,
value path and trace position, each component present exactly as
`ix://agent-ix/quire-specification/FR-351` states for the protocol family —
imported byte-exactly from the accepted family-neutral separating-witness record
carried on the protocol-analysis record of `native-run-result/2`
(`ix://agent-ix/quire-specification/FR-352` "Protocol-analysis result") that
QSL's `protocol-analysis` engine writes, under the
`quire.protocol.result/v1-draft.2` contract version.

## Inputs

One settled `realizability` or `refinement` claim whose settlement basis is
`decisive-witness` or `decisive-counterexample`, and the admitted
`ix://agent-ix/quire-specification/FR-351` separating-witness record carried on
`native-run-result/2` for that settlement.

## Outputs

One canonical result whose settlement carries exactly one trace-position
witness when the basis is decisive, and none when it is not. The witness carries
its deciding element identity, its value path, and the trace position the
protocol family retains under `ix://agent-ix/quire-specification/FR-243`; it
carries an index exactly where the admitted record assigns one, and that
component compares only to absence where the record assigns none.

## Behavior

When settlement basis is `decisive-witness` or `decisive-counterexample` for a
`realizability` or `refinement` claim, the result producer SHALL carry the
imported deciding-element identity, value path and trace position, and the index
where the admitted record assigns one, from the admitted separating-witness
record on `native-run-result/2`.

A settlement whose basis is `closed-scope`, `unsettled` or `unavailable`
carries no witness. A decisive basis without its witness refuses the result.

Each carried component is copied byte-exactly from the admitted record, and a
component the record leaves absent is carried as absence. Normalizing,
recomputing, reordering, substituting a default for an absent component, or
supplying a display index refuses the result rather than repairing it, exactly
as `ix://agent-ix/quire-specification/FR-351`-AC-5 requires of a strict reader.

A settlement whose analysis backend returns no `native-run-result/2`
protocol-analysis record for a decisive result settles `unsupported` with a
warning naming the required capability, and carries no substitute witness. QSL's
own engine writes that record for every `proved`, `refuted`, `inconclusive` and
`unsupported` result, so its decisive results always carry their witness
(FR-150-AC-17); this case is a backend that serves the kind without writing the
record, such as a `refinement` backend, which FR-352 gives no protocol-analysis
record.

The carrying contract version is `quire.protocol.result/v1-draft.2`. A producer
selects that identifier explicitly at production time and fixes it in the
produced artifact; a consumer that requires the witness record selects and
regenerates against `v1-draft.2`. This repository states the identifier under
`ix://agent-ix/quire-specification/FR-352`'s cross-repo note, which assigns the
`quire.protocol.result` draft-revision bump to `agent-ix/quire-protocol#96`;
the shape it carries stays `ix://agent-ix/quire-specification/FR-351`'s, and the
`ix://agent-ix/quire-specification/FR-240` members it carries are otherwise
unchanged.

QSL writes and reads the record (`ix://agent-ix/quire-specification/FR-352`); this
repository carries the witness from it, byte-exactly, and states the contract
version the carried result is produced under.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-302-AC-1 | Every `realizability` or `refinement` claim settled `decisive-witness` or `decisive-counterexample` carries exactly one trace-position witness whose deciding-element identity, value path and trace position byte-match the admitted separating-witness record, and whose index byte-matches that record where it assigns one. | Test (TC-148) |
| FR-302-AC-2 | A settlement basis outside `decisive-witness`/`decisive-counterexample` carries no witness, and a decisive basis missing its witness refuses the result. | Test (TC-148) |
| FR-302-AC-3 | Each carried component is copied byte-exactly from the admitted record and an absent component is carried as absence; normalizing, recomputing, reordering, substituting a default for absence, or supplying a display index refuses the result, exactly as `ix://agent-ix/quire-specification/FR-351`-AC-5 requires. | Test (TC-148) |
| FR-302-AC-4 | A decisive settlement whose analysis backend returns no `native-run-result/2` protocol-analysis record settles `unsupported` with a warning naming the required capability and carries no witness, while QSL's own engine, which writes the record, settles its decisive `realizability` result with the witness (FR-150-AC-17). | Test (TC-148) |
| FR-302-AC-5 | The witness-bearing result carries contract version `quire.protocol.result/v1-draft.2`, selected explicitly at production and fixed in the artifact, and a consumer requiring the witness regenerates against that version. | Test (TC-148) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Separating-witness record | `ix://agent-ix/quire-specification/FR-351` | value, keyed (deciding element identity, index where assigned, value path, trace position where retained) |
| Witness-bearing native wire | `ix://agent-ix/quire-specification/FR-352` (`native-run-result/2`) | value |
| Settlement basis | `ix://agent-ix/quire-specification/FR-243` | value |
| Canonical protocol result members | `ix://agent-ix/quire-specification/FR-240` | value |

### Accepted owner rulings

`agent-ix/quire-specification#114` mints `ix://agent-ix/quire-specification/FR-351`
as the family-neutral separating-witness record shared by codegen,
quire-verification and quire-protocol consumers, and
`ix://agent-ix/quire-specification/FR-352` as the `native-run-result/2` wire that
carries it; exhausted and cancelled settlements carry no witness, and an
`unsupported` settlement emits no substitute witness. FR-352's cross-repo note
assigns this repository's own `quire.protocol.result` draft-revision bump to
`agent-ix/quire-protocol#96`, the ticket this requirement lands under. This
requirement defines this repository's carrier for that record and mints no
shape of its own; `agent-ix/quire-protocol#80` lands the physical schema and the
producer and reader code.

## Dependencies

- [FR-006](./FR-006-produce-canonical-protocol-results.md),
  [FR-031](./FR-031-canonical-result-identity-and-digest.md) and
  [FR-032](./FR-032-retain-orthogonal-result-axes.md) govern the
  `v1-draft.1` canonical result this requirement extends.
- `ix://agent-ix/quire-specification/FR-240` supplies the canonical protocol
  result members this requirement leaves otherwise unchanged.
- `ix://agent-ix/quire-specification/FR-290` names `realizability` and
  `refinement` as the claim kinds whose decisive settlements carry a witness.
- `ix://agent-ix/quire-specification/FR-351` supplies the separating-witness
  record shape and component-presence rules this requirement carries, and
  `ix://agent-ix/quire-specification/FR-352` the `native-run-result/2` wire it is
  imported from; both are minted by `agent-ix/quire-specification#114`.
- `agent-ix/quire-protocol#80` lands the `v1-draft.2` schema and the producer and
  reader code this requirement specifies.
