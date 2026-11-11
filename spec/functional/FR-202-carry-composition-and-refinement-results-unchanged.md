---
id: FR-202
title: "Carry composition and refinement results without reinterpretation"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-030, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-200, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-301, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-241, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-242, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-177, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-148, type: references }
  - { target: ix://agent-ix/quire-spec-language/FR-530, type: depends_on }
---
# FR-202: Carry composition and refinement results without reinterpretation

## Description

When a composition or refinement backend returns a result, the library SHALL carry the paired counterexample, certificate, discharge record, witness or cause in the disposition with the same carry-unchanged semantics FR-301-AC-6 states for refinement. No outcome value is derived from what it carries.

## Inputs

A returned composition result (FR-200) or refinement result (FR-301): a certificate, a paired counterexample, an incompatibility witness, a discharge record, or a refusal, `unsupported` or other non-success cause, together with the assessment-execution and truth values the result itself states.

## Outputs

The disposition carrying every returned artifact byte-equal and its axes as the backend stated them.

## Behavior

- The library shall carry each returned certificate, counterexample, witness and discharge record byte-equal to what the backend returned, as FR-301-AC-6 requires of a refinement result.
- When a refinement backend returns a paired counterexample, the library shall carry both of its halves, the implementation side and the specification side, byte-equal.
- The library shall reconstruct no counterexample half that the backend did not return.
- The library shall carry a returned refusal or non-success cause unchanged and without mapping it to another cause.
- The library shall carry the assessment-execution and truth values of `ix://agent-ix/quire-specification/FR-241` and `ix://agent-ix/quire-specification/FR-242` exactly as the backend's result states them.
- The library shall derive no truth value from the presence, absence or content of a certificate, counterexample, witness or discharge record.
- The library shall set the assessment-execution value `refused` on no composition or refinement disposition, because that value states that the evaluator refused the request.
- The library shall carry a non-success result in no form that presents it as satisfied.

A non-success result includes an unsupported dynamic behaviour or a missing correspondence (`ix://agent-ix/quire-specification/FR-177`), a premise left unproved, and a backend-reported strengthened assumption (QSL FR-148's `protocol-assumption-not-established`). Each is carried by its own name. A refusal the library itself raises on a disposition under FR-030 is a separate act from carrying a backend's value, and it sets no execution value.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-202-AC-1 | Given sentinel results from a stand-in for the QSL call boundary, a composition result, discharge record and incompatibility witness appear in the disposition byte-equal to the sentinels. | Test (TC-202) |
| FR-202-AC-2 | Given a stand-in refinement counterexample with an implementation half and a specification half, and a stand-in simulation certificate, the disposition carries both halves and the certificate byte-equal. | Test (TC-202) |
| FR-202-AC-3 | Given a stand-in counterexample with only one half, the disposition carries that half and no second half appears. | Test (TC-202) |
| FR-202-AC-4 | Given stand-in results carrying `unsupported-requested-capability`, `premise-unproved`, `protocol-assumption-not-established` and a cause name the library does not recognise, each cause appears in its disposition unchanged, none is mapped to another, and none carries a success value. | Test (TC-202) |
| FR-202-AC-5 | Given a stand-in result whose assessment-execution is not `completed` and which also carries certificate-shaped bytes, the disposition carries the stated execution value, carries no truth value, and derives none from the bytes. | Test (TC-202) |
| FR-202-AC-6 | Over every stand-in outcome in this table, no composition or refinement disposition carries assessment-execution `refused` unless the stand-in's result stated it, and a stand-in that states it is carried as stated. | Test (TC-202) |
| FR-202-AC-7 | Given a stand-in result for unsupported dynamic behaviour or a missing correspondence, no disposition presents `satisfied`, and the cause is carried unchanged. | Test (TC-202) |

## Dependencies

- [FR-301](./FR-301-admit-protocol-refinement-dispatch.md) supplies FR-301-AC-6, whose carry-unchanged semantics this requirement extends to composition artifacts and to the paired counterexample's halves.
- [FR-200](./FR-200-dispatch-composition-claims-to-qsl.md) supplies the composition dispatch whose results this requirement carries.
- [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) supplies the byte-equality rule for producer-authored components (FR-030-AC-6).
- [FR-008](./FR-008-return-typed-refusals.md) supplies the unknown-source rule for a cause the library does not recognise.
- `ix://agent-ix/quire-specification/FR-241` and `ix://agent-ix/quire-specification/FR-242` own the execution and truth vocabularies; `ix://agent-ix/quire-specification/FR-177` and QSL FR-148, FR-530 own the results carried.
