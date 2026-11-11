---
id: FR-152
title: "Carry a QSL projection or realizability witness joined to the linked graph"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-002, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-030, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-150, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-302, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-243, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-351, type: depends_on }
---
# FR-152: Carry a QSL projection or realizability witness joined to the linked graph

## Description

When QSL's backend returns a witness or counterexample for a `realizability` claim, or a projected local artifact for a `local-projection` claim, the result producer SHALL carry it unchanged, joined to the FR-002 linked graph through the source identities the disposition retains under FR-030.

## Inputs

One disposition from FR-150 whose backend result carries a `realizability` witness or counterexample, or a `local-projection` projected artifact, and the FR-002 linked graph.

## Outputs

The disposition carrying the backend's witness, counterexample or projected local artifact unchanged beside the source identities it retains under FR-030; for a `realizability` claim settled on `decisive-witness` or `decisive-counterexample`, the trace-position witness under FR-302; or one FR-030 refusal of that disposition.

## Behavior

- The result producer shall carry the backend's witness, counterexample or projected local artifact unchanged.
- The result producer shall carry the trace-position witness of a `realizability` claim settled `decisive-witness` or `decisive-counterexample` as FR-302 states.
- The result producer shall join the carried trace position to the linked graph through the source identities the disposition retains under FR-030.
- The result producer shall interpret the trace position as no graph identity itself.
- When the disposition retains no source identity, the result producer shall refuse it under FR-030.
- The result producer shall retain the derived-artifact correspondence FR-030 requires beside the source identities.
- The result producer shall compute no witness, no minimisation and no reordering of a witness.
- The result producer shall leave every independent disposition unchanged when it refuses one.

`ix://agent-ix/quire-specification/FR-351` types the deciding element as the exact deciding value and the trace position as a position within the retained trace, and maps neither to a graph node. This requirement therefore defines no graph-resolution rule for a witness component and adds no component to FR-302's import.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-152-AC-1 | Given a stand-in returning a `realizability` result settled `decisive-witness`, the disposition carries the trace-position witness per FR-302 byte-for-byte and retains the claim's source identities. | Test (TC-162) |
| FR-152-AC-2 | Given a stand-in returning a `realizability` result settled `decisive-counterexample`, the disposition carries the trace-position witness byte-for-byte and retains the claim's source identities. | Test (TC-162) |
| FR-152-AC-3 | Given a stand-in returning a deliberately non-minimal witness, the disposition carries it byte-for-byte, with no step dropped, merged or reordered. | Test (TC-162) |
| FR-152-AC-4 | Given a decisive request whose disposition retains no source identity, that disposition is refused under FR-030, and every independent disposition is unchanged. | Test (TC-162) |
| FR-152-AC-5 | Given a stand-in returning a projected local artifact for a `local-projection` claim, the disposition carries it unchanged. | Test (TC-162) |
| FR-152-AC-6 | Given a projected local artifact whose identity disagrees with the one derived from its selected tuple, that disposition is refused as FR-030-AC-5 requires. | Test (TC-162) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Settlement basis | `ix://agent-ix/quire-specification/FR-243` | value |
| Separating-witness record | `ix://agent-ix/quire-specification/FR-351` | value |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes that the library carries QSL's witness and joins it to source identities.
- [FR-002](./FR-002-link-finite-protocol-representation.md) supplies the linked graph the identities are joined to.
- [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) supplies the disposition-scoped refusal and derived-artifact correspondence.
- [FR-150](./FR-150-dispatch-projection-and-realizability-to-qsl.md) supplies the disposition whose result is carried.
- [FR-302](./FR-302-carry-trace-position-witness.md) supplies the trace-position witness this requirement joins to the linked graph.
- `ix://agent-ix/quire-specification/FR-243` and `ix://agent-ix/quire-specification/FR-351` supply the settlement basis and the separating-witness record.
