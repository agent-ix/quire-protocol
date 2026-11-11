---
id: FR-201
title: "Bind composition dispositions to their premise lists and component subjects"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-030, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-200, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: references }
  - { target: ix://agent-ix/quire-specification/FR-489, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-530, type: depends_on }
---
# FR-201: Bind composition dispositions to their premise lists and component subjects

## Description

When a composition disposition is returned, the dispatcher SHALL keep the request's environment, rely, liveness-dependency and fairness premises as four separate lists bound to their component subjects. The dispatcher SHALL refuse a disposition whose retained lists or subjects differ from the request's.

## Inputs

One composition request (FR-200) carrying the component subjects (checked package and subject identity each) and the four premise lists of `ix://agent-ix/quire-specification/FR-489`, and the backend's returned result with its discharge record.

## Outputs

The composition disposition retaining the four lists and the subjects exactly as requested, with the backend's discharge record bound to them, or one refusal of that disposition under FR-030 naming the list or subject that differs.

## Behavior

- The dispatcher shall pass the `environment`, `relies`, `liveness_dependencies` and `fairness` lists to the backend as four separate lists, each entry byte-equal to the request's.
- The dispatcher shall retain all four lists in the disposition, collapsing none into a flag and merging none into another.
- The dispatcher shall carry the discharge record as the backend returned it, with one entry per discharged slot and each entry's role as QSL names it, and shall classify no slot into a premise list.
- When a returned result names a component subject that is not among the request's subjects, the dispatcher shall refuse the disposition under FR-030 with the cause `identity-mismatch`.
- When the backend returns a slot undischarged, the disposition shall retain that slot as undischarged.
- The dispatcher shall mark no slot discharged on any basis other than the backend's discharge record.

Which guarantee discharges which premise is QSL's decision (QSL FR-530, `ix://agent-ix/quire-specification/FR-489` "Discharge"). This requirement binds the answer to the question that was asked; it decides none of it.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-201-AC-1 | A recording stand-in for the QSL call boundary receives the four premise lists as four separate lists whose entries are byte-equal to the request's, and no Boolean flag stands in for any list. | Test (TC-201) |
| FR-201-AC-2 | The disposition retains all four lists exactly as requested, and dropping or merging any one of them from the disposition refuses it under FR-030 without changing an independent disposition. | Test (TC-201) |
| FR-201-AC-3 | Given a stand-in discharge record whose entries carry the roles QSL names, including `component-guarantee` and `fixed-point` entries, the disposition carries every entry byte-equal, refuses none of them, and the library derives no slot, role or premise list of its own. | Test (TC-201) |
| FR-201-AC-4 | A stand-in result naming a component subject absent from the request refuses the disposition with `identity-mismatch`, and the refusal names the subject. | Test (TC-201) |
| FR-201-AC-5 | Given a request whose supplied guarantees would discharge a rely slot by a local reading and a stand-in that returns that slot undischarged, the disposition retains the slot as undischarged and carries no success. | Test (TC-201) |
| FR-201-AC-6 | Given a stand-in discharge record that is empty, no slot in the disposition is marked discharged. | Test (TC-201) |

## Dependencies

- [FR-200](./FR-200-dispatch-composition-claims-to-qsl.md) supplies the composition request this requirement binds.
- [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) supplies the premise-binding refusal this requirement applies to the four lists and the subjects.
- [FR-008](./FR-008-return-typed-refusals.md) supplies the closed cause vocabulary that holds `identity-mismatch`.
- `ix://agent-ix/quire-specification/FR-489` and QSL FR-530 define the four lists, the slots and the discharge record this requirement binds but does not restate.
