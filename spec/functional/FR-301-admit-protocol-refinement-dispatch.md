---
id: FR-301
title: "Admit protocol refinement dispatch"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/AD-001
    type: references
  - target: ix://agent-ix/quire-protocol/FR-002
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-005
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-029
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-030
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-060
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-176
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-177
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-290
    type: references
  - target: ix://agent-ix/quire-specification/FR-292
    type: references
  - target: ix://agent-ix/quire-specification/AD-007
    type: references
---
# FR-301: Admit protocol refinement dispatch

## Description

When a `refinement` claim is requested and the accepted assume-guarantee and
refinement contracts (`ix://agent-ix/quire-specification/FR-176`,
`ix://agent-ix/quire-specification/FR-177`) are admitted, the dispatcher SHALL
route it to QSL's refinement backend over the FR-002 linked graph's source
identities and carry that backend's result, rather than treating it as an
advanced claim with no accepted backend.

## Inputs

One requested `refinement` claim with its admitted fragment, selected backend,
assumptions and source identities; the FR-002 linked participant/role/channel
graph for the paired implementation and specification protocols; and the
admitted FR-176/FR-177 assume-guarantee and refinement contracts.

## Outputs

One source-bound refinement disposition retaining its exact admitted fragment,
backend, assumptions and source/derived-artifact correspondence exactly as
FR-030 requires every disposition to, or one explicit `unsupported` disposition
when no admitted backend can discharge the claim.

## Behavior

The dispatcher SHALL send an admitted refinement claim to QSL's refinement
backend, which owns the explicit-state product construction (paired
correspondence between implementation and specification states, the relation
FR-177 selects, and assumption discharge under FR-176), and SHALL carry its
result, counterexample or certificate unchanged
([ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md)).

A refinement disposition SHALL NOT carry the complete-global-conformance
premise set as evidence for its own success, exactly as
`ix://agent-ix/quire-specification/FR-290`-AC-3 requires of every non-global
claim.

A monitorable protocol whose refinement backend is absent or unaccepted SHALL
retain its independent global conformance result and return an explicit
`unsupported` refinement disposition, mirroring the projection-absence pattern
`ix://agent-ix/quire-specification/FR-060`-AC-1 states for monitorable-but-
unprojectable protocols.

A refinement request without an accepted FR-292 analysis backend settles
`unsupported` with a warning naming the required capability.

This requirement extends FR-005's closed dispatch and FR-029/FR-030's
independence and premise-binding discipline to the refinement claim kind; it
introduces no separate dispatch path, resource guard or fallback backend.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-301-AC-1 | A refinement disposition never carries the complete-global-conformance premise set as evidence for its own success, exactly as `ix://agent-ix/quire-specification/FR-290`-AC-3 requires. | Test (TC-147) |
| FR-301-AC-2 | A monitorable protocol whose refinement backend is absent or unaccepted retains its independent global conformance result and returns an explicit `unsupported` refinement disposition, exactly as `ix://agent-ix/quire-specification/FR-060`-AC-1's projection-absence pattern requires. | Test (TC-147) |
| FR-301-AC-3 | Once FR-176 and FR-177 are admitted, a requested refinement claim is dispatched to QSL's refinement backend and its result carried unchanged, bound to the FR-002 linked graph's source identities; this library builds no product construction, separate refinement engine or local relation definition. | Test (TC-147) |
| FR-301-AC-4 | A refinement request without an accepted FR-292 analysis backend settles `unsupported` with a warning naming the required capability, and this settlement is never a hold, blocker or refusal error. | Test (TC-147) |
| FR-301-AC-5 | A refinement disposition retains its exact admitted fragment, backend, assumptions, communication/visibility/environment premises and source/derived-artifact correspondence exactly as [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) requires of every disposition. | Test (TC-147) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Refinement claim kind | `ix://agent-ix/quire-specification/FR-290` | value |
| Assume-guarantee composition | `ix://agent-ix/quire-specification/FR-176` | value |
| Refinement relation | `ix://agent-ix/quire-specification/FR-177` | value |
| Analysis backend | `ix://agent-ix/quire-specification/FR-292` | authored |

### Accepted owner rulings

`ix://agent-ix/quire-specification/FR-176` and
`ix://agent-ix/quire-specification/FR-177` are the accepted assume-guarantee and
refinement owner contracts, landed and owned by `agent-ix/quire-specification`;
QSL implements them. This requirement consumes those contracts
directly and defines no local refinement relation, correspondence rule or
assumption-discharge algorithm; the explicit-state product construction is
QSL's (ADR-010), and this repository owns only the dispatch and carrying.

## Dependencies

- [FR-002](./FR-002-link-finite-protocol-representation.md) supplies the linked graph
  whose source identities bind the dispatched claim.
- [FR-005](./FR-005-dispatch-independent-protocol-claims.md) supplies closed
  claim dispatch.
- [FR-029](./FR-029-return-one-disposition-per-requested-claim.md) and
  [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) supply
  independence and premise binding.
- `ix://agent-ix/quire-specification/FR-060` states the projection-absence
  pattern this requirement mirrors for refinement.
- `ix://agent-ix/quire-specification/FR-176` and
  `ix://agent-ix/quire-specification/FR-177` supply the assume-guarantee and
  refinement contracts this requirement dispatches
  to QSL's backend.
- `ix://agent-ix/quire-specification/FR-292` supplies the analysis backend whose
  accepted capability selects a refinement check.
