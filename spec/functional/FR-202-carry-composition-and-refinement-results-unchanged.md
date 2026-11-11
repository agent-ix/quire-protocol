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
  - { target: ix://agent-ix/quire-specification/FR-489, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-360, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-243, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-148, type: references }
  - { target: ix://agent-ix/quire-spec-language/FR-530, type: depends_on }
---
# FR-202: Carry composition and refinement results without reinterpretation

## Description

When a composition or refinement backend returns a result, the library SHALL carry the paired counterexample, certificate, discharge record, witness or cause in the disposition with the same carry-unchanged semantics FR-301-AC-6 states for refinement. The library SHALL derive no outcome value from what it carries.

## Inputs

A returned composition result (FR-200) or refinement result (FR-301): a certificate, a paired counterexample, an incompatibility witness, a discharge record, or a refusal, `unsupported` or other non-success cause, together with the assessment-execution and truth values the result itself states.

## Outputs

The disposition carrying every returned artifact byte-equal and its axes as the backend stated them.

## Behavior

- The library shall carry each returned certificate, counterexample, witness and discharge record byte-equal to what the backend returned, as FR-301-AC-6 requires of a refinement result.
- When a refinement backend returns a paired counterexample, the library shall carry both of its halves, the implementation side and the specification side, byte-equal.
- The library shall reconstruct no counterexample half that the backend did not return.
- The library shall carry a returned refusal or non-success cause unchanged and without mapping it to another cause.
- The library shall carry the result label QSL's backend returns (for composition `proved`, `inconclusive`, `unsupported` or `incomplete`) exactly as returned.
- The library shall read a returned composition label into the assessment-execution and truth values of `ix://agent-ix/quire-specification/FR-241` and `ix://agent-ix/quire-specification/FR-242` only through the owner table of `ix://agent-ix/quire-specification/FR-360`, which `ix://agent-ix/quire-specification/FR-489` makes the composition result's vocabulary.
- The library shall read the settlement basis of `ix://agent-ix/quire-specification/FR-243` from the Settlement table of `ix://agent-ix/quire-specification/FR-489` (`proved` to `decisive-witness`, `inconclusive` to `unsettled`, `unsupported` and `incomplete` to `unavailable`), because FR-360 leaves `proved` open between two bases.
- The library shall derive no execution or truth value for a label the FR-360 table does not contain.
- The library shall carry such a label unchanged.
- The library shall derive no truth value from the presence, absence or content of a certificate, counterexample, witness or discharge record.
- The library shall set the assessment-execution value `refused` on no composition or refinement disposition on its own, because that value states that the evaluator refused the request.
- When QSL's backend returns a refusal of the composition or refinement item (for example `malformed-declaration` for a cycle with no profile), the library shall carry that refusal unchanged, and the owner's `refused` reading applies to it.
- The library shall carry a non-success result in no form that presents it as satisfied.

Refinement has no owner table from `ix://agent-ix/quire-specification/FR-177`'s outputs (refines, violates, inconclusive, unsupported, failed) to the FR-241, FR-242 and FR-243 values. This requirement therefore carries a refinement result's label and artifacts and derives no axis value for it. The missing mapping is an open question for the owner of `quire-specification`, not a rule invented here. A second open question for that owner is that the FR-489 Settlement table lists `incomplete` (basis `unavailable`) as an FR-360 label, while FR-360 has no `incomplete` member, so no execution or truth value is derivable for it.

A non-success result includes an unsupported dynamic behaviour or a missing correspondence (`ix://agent-ix/quire-specification/FR-177`), a premise left unproved, and a backend-reported strengthened assumption (QSL FR-148's `protocol-assumption-not-established`). Each is carried by its own name. A refusal the library itself raises on a disposition under FR-030 is a separate act from carrying a backend's value, and it sets no execution value.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-202-AC-1 | Given sentinel results from a stand-in for the QSL call boundary, a composition result, discharge record and incompatibility witness appear in the disposition byte-equal to the sentinels. | Test (TC-202) |
| FR-202-AC-2 | Given a stand-in refinement counterexample with an implementation half and a specification half, and a stand-in simulation certificate, the disposition carries both halves and the certificate byte-equal. | Test (TC-202) |
| FR-202-AC-3 | Given a stand-in counterexample with only one half, the disposition carries that half and no second half appears. | Test (TC-202) |
| FR-202-AC-4 | Given stand-in results carrying `unsupported-requested-capability`, `premise-unproved`, `protocol-assumption-not-established` and a cause name the library does not recognise, each cause appears in its disposition unchanged, none is mapped to another, and none carries a success value. | Test (TC-202) |
| FR-202-AC-5 | Given a stand-in composition result labelled `unsupported` that also carries certificate-shaped bytes, the disposition carries the label, reads it as execution `unsupported` and truth `unavailable` through the FR-360 table and basis `unavailable` through the FR-489 table, and derives no truth from the bytes; given `proved` and `inconclusive`, they read as execution `completed` with truth `satisfied` and `pending`, and basis `decisive-witness` and `unsettled`; given `incomplete`, the label is carried unchanged with basis `unavailable` and no derived execution or truth value. | Test (TC-202) |
| FR-202-AC-6 | Over every stand-in outcome in this table, no composition or refinement disposition carries execution `refused` unless the stand-in returned a refusal of the item; a stand-in `malformed-declaration` refusal is carried unchanged, and a stand-in refinement result of any label receives no derived axis value. | Test (TC-202) |
| FR-202-AC-7 | Given a stand-in result for unsupported dynamic behaviour or a missing correspondence, no disposition presents `satisfied`, and the cause is carried unchanged. | Test (TC-202) |

## Dependencies

- [FR-301](./FR-301-admit-protocol-refinement-dispatch.md) supplies FR-301-AC-6, whose carry-unchanged semantics this requirement extends to composition artifacts and to the paired counterexample's halves.
- [FR-200](./FR-200-dispatch-composition-claims-to-qsl.md) supplies the composition dispatch whose results this requirement carries.
- [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) supplies the byte-equality rule for producer-authored components (FR-030-AC-6).
- [FR-008](./FR-008-return-typed-refusals.md) supplies the unknown-source rule for a cause the library does not recognise.
- `ix://agent-ix/quire-specification/FR-489` and `ix://agent-ix/quire-specification/FR-360` own the composition result label and its reading into the axes; `ix://agent-ix/quire-specification/FR-241`, `ix://agent-ix/quire-specification/FR-242` and `ix://agent-ix/quire-specification/FR-243` own the axis vocabularies; `ix://agent-ix/quire-specification/FR-177` and QSL FR-148, FR-530 own the results carried.
