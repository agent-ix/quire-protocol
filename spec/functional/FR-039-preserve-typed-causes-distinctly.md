---
id: FR-039
title: "Preserve every typed cause and disposition distinctly"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/FR-008
    type: part_of
  - target: ix://agent-ix/quire-protocol/FR-014
    type: depends_on
  - target: ix://agent-ix/quire-protocol/FR-036
    type: depends_on
  - target: ix://agent-ix/quire-protocol/NFR-002
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-272
    type: references
  - target: ix://agent-ix/quire-specification/FR-270
    type: references
---
# FR-039: Preserve every typed cause and disposition distinctly

## Description

When a source response carries typed causes, the library SHALL retain every one of them distinctly in the response's own declared semantic order.
The library SHALL keep the cause pairs enumerated in Behavior below structurally distinct from each other.
The library SHALL NOT let an exhausted or zero diagnostic-detail budget erase a cause's existence.
The library SHALL NOT report overall success when any retained cause or disposition is present.

## Inputs

One typed upstream failure response carrying one or more typed causes or
dispositions in a declared semantic order, under a diagnostic-detail budget that
may be zero.

## Outputs

The envelope's ordered typed-cause list, containing every cause and disposition
the response declared, in the response's declared semantic order, with the
structurally distinct pairs below unmerged.

## Behavior

The library SHALL keep these local cause pairs structurally distinct: missing versus conflicting definitions; wrong anchors versus missing observations; unproved definedness versus logical violations; identity mismatch versus incomplete evidence; and unsupported capability versus projection finding loss.
The library SHALL keep the independently typed resource-incomplete outcome distinct from every refusal without representing it as a refusal cause.
The library SHALL retain every independent cause and disposition a response declares in that response's declared semantic order.
The library SHALL NOT drop, merge or reorder a retained cause to fit a diagnostic-detail budget.
The library SHALL represent a zero diagnostic-detail budget as a suppressed rendering of causes that exist, never as an absence of causes and never as success.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-039-AC-1 | The missing/conflicting-definition, wrong-anchor/missing-observation, unproved-definedness/logical-violation, identity-mismatch/incomplete-evidence, and unsupported-capability/projection-finding-loss cause pairs each remain structurally distinct, and merging either member of any pair into the other fails. | Test (TC-011) |
| FR-039-AC-2 | A response declaring independent causes or dispositions yields an envelope retaining all of them in the response's declared semantic order; dropping one, reordering two, or merging two into one fails. | Test (TC-011) |
| FR-039-AC-3 | Simultaneous independent invalid-input and missing-input dispositions both remain represented under a zero diagnostic-detail control, and that control cannot produce overall success. | Test (TC-011) |
| FR-039-AC-4 | Two distinct refusal axes whose retained typed-cause sets are equal fail FR-014-AC-1's pairwise check, so collapsing any pair in FR-039-AC-1 into a single cause also fails the injectivity check rather than reducing detail. | Test (TC-011) |
| FR-039-AC-5 | A resource-incomplete outcome and an unsupported semantic disposition are never reported as each other or as a `ProtocolRefusal`, and neither is reported as a completed assessment whose finding set is empty. | Test (TC-011, IT-003) |
| FR-039-AC-6 | No local payload field name, total order or spelling convention is authored for a typed cause; the retained order is the response's declared semantic order and this requirement establishes none of its own. | Test (TC-011) |

## Adopted vocabulary

Local typed causes are FR-008's closed `quire.protocol/refusal/1` members. Source
causes are retained from their exact selected contracts. The resource-incomplete
outcome is [NFR-002](../non-functional/NFR-002-enforce-resource-bounds.md)'s and
the completed-with-empty-findings outcome is
[FR-006](./FR-006-produce-canonical-protocol-results.md)'s;
[FR-014](./FR-014-typed-outcome-identity.md) holds the three outcome kinds
non-equivalent.

The typed cause is one of the two producer-supplied interior components
FR-014-AC-1 compares across a pair of distinct refusal axes. Merging a cause pair
therefore does not merely lose detail: it removes the signal injectivity is
computed from, which is why FR-039-AC-4 states the consequence as a failing check
rather than as a tolerable simplification.

Local cause order is the order the Protocol operation emits under its owning
requirement. Source cause order is the response's declared semantic order. If a
selected source contract defines no order, Protocol retains wire order and marks
the ordering authority as `source-wire`; it never invents a semantic sort key.

## Dependencies

- [FR-008](./FR-008-return-typed-refusals.md) owns the envelope; this requirement
  is `part_of` it.
- [FR-036](./FR-036-carry-complete-refusal-envelope.md) obliges the ordered
  typed-cause field to be present; this requirement obliges its content.
- [NFR-002](../non-functional/NFR-002-enforce-resource-bounds.md) owns the
  resource-incomplete outcome that FR-039-AC-5 keeps distinct.
- [FR-014](./FR-014-typed-outcome-identity.md) owns the injectivity obligation and
  the three-way outcome distinction.

## Status

Draft. The common-envelope implementation is partial; Task-006 completes the
remaining paths and TC-011.
