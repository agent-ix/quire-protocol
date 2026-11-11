---
id: FR-203
title: "Carry declared fixed-point cycles as typed unsupported"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-029, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-057, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-200, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-202, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-176, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-491, type: references }
  - { target: ix://agent-ix/quire-spec-language/FR-530, type: depends_on }
---
# FR-203: Carry declared fixed-point cycles as typed unsupported

## Description

When a composition request declares a dependency cycle and QSL reports the cycle's fixed-point discharge as an `unsupported` request, the library SHALL carry that outcome as the FR-057 typed `unsupported` disposition with QSL's cause name. No guarantee in the cycle is available to discharge another premise.

## Inputs

A composition request (FR-200) whose declaration contains a strongly connected component of qualified guarantees (`ix://agent-ix/quire-specification/FR-176` "Discharge fixed point", `ix://agent-ix/quire-specification/FR-491`), and the result the QSL composition backend returns for it.

## Outputs

One `unsupported` disposition under FR-057's pattern carrying QSL's `unsupported-requested-capability` cause name unchanged and the cycle's checked node ids, or the backend's other returned outcome carried unchanged under FR-202.

## Behavior

- When the backend reports the cycle's fixed-point discharge as an unsupported request, the library shall return the FR-057 `unsupported` disposition carrying `unsupported-requested-capability`.
- The library shall define no second name or cause for the unserved cycle.
- When the backend returns an outcome other than `unsupported` for the cycle, such as `premise-unproved` or a `malformed-declaration` refusal, the library shall carry that outcome unchanged under FR-202.
- The library shall not remap such an outcome to `unsupported`.
- The library shall make no guarantee of the cycle available as an assumption to discharge any premise, in the request or in the retained disposition.
- The library shall compute no iteration, lattice element or least fixed point.
- The library shall leave every other claim's disposition unchanged (FR-029).

This requirement carries the outcome of QSL's gap for declared least-fixed-point cycles. A cycle result with recorded iterations, once QSL returns one, is a returned artifact carried unchanged by FR-202; this requirement needs no change for it.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-203-AC-1 | Given a declared two-guarantee cycle and a stand-in for the QSL call boundary that reports `unsupported-requested-capability`, the disposition is `unsupported`, carries that cause name unchanged and the cycle's node ids, and carries no success value. | Test (TC-203) |
| FR-203-AC-2 | Given a stand-in returning `premise-unproved`, and another returning a `malformed-declaration` refusal for the cycle, each appears unchanged and neither is mapped to `unsupported`. | Test (TC-203) |
| FR-203-AC-3 | Given the cycle request, the recording stand-in receives one composition request item for the composition claim and no other call, and the retained disposition lists no cycle guarantee among its assumptions. | Test (TC-203) |
| FR-203-AC-4 | Given a stand-in that supplies a proved guarantee for one member of the cycle and `unsupported-requested-capability` for the cycle, the other member's premise is not marked discharged and the disposition is not success. | Test (TC-203) |
| FR-203-AC-5 | Given the cycle claim `unsupported` beside an acyclic composition claim the stand-in proves, the acyclic disposition equals the one returned without the cycle claim. | Test (TC-203) |
| FR-203-AC-6 | The `unsupported` disposition for the cycle introduces no cause name beyond those FR-057 carries from QSL. | Test (TC-203) |

## Dependencies

- [FR-057](./FR-057-map-qsl-unsupported-outcomes-to-typed-dispositions.md) supplies the cause-carrying pattern, the `unsupported-requested-capability` name and the rule that the library defines no second name.
- [FR-200](./FR-200-dispatch-composition-claims-to-qsl.md) supplies the composition dispatch the cycle is part of, and [FR-202](./FR-202-carry-composition-and-refinement-results-unchanged.md) the carrying of every other cycle outcome.
- [FR-029](./FR-029-return-one-disposition-per-requested-claim.md) supplies the independence of the other claims.
- `ix://agent-ix/quire-specification/FR-176` and `ix://agent-ix/quire-specification/FR-491` define the cycle and its fixed-point rule; QSL FR-530 owns the slot that holds the fixed-point result.
