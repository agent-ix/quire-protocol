---
id: FR-204
title: "Refuse a refinement result whose assumptions are stronger than the request's"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-029, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-030, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-040, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-202, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-301, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-177, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-148, type: references }
---
# FR-204: Refuse a refinement result whose assumptions are stronger than the request's

## Description

When a refinement backend returns a result that retains an assumption identity absent from the request's assumptions, the library SHALL refuse that refinement disposition, because the result was established under assumptions stronger than the ones requested.

## Inputs

A requested `refinement` claim (FR-301) with its assumption identities and states (FR-030-AC-4), and the backend's returned result with the assumption identities it retains.

## Outputs

The disposition carried under FR-202 when every retained assumption identity is among the request's, or one refusal of that disposition under FR-030 naming each extra assumption identity.

## Behavior

- The library shall compare the returned result's retained assumption identities with the request's by exact identity.
- When a retained assumption identity is absent from the request's assumptions, the library shall refuse the refinement disposition with the cause `identity-mismatch` naming that identity.
- When the returned result omits its retained assumption identities, the library shall refuse the refinement disposition with the cause `identity-mismatch` naming the absent field, because absence compares only to absence (FR-030-AC-4).
- When every retained assumption identity is among the request's, the library shall carry the result, whether or not the request holds assumptions the result did not use.
- When the refused disposition would have been success, the library shall emit no success value for it (FR-040).
- The library shall leave every independent disposition unchanged (FR-029).
- The library shall not replace the refused result with a fallback backend's (FR-030).
- When the backend itself reports a strengthened assumption as a non-success finding, the library shall carry that finding unchanged under FR-202 and not refuse it.

The library decides inclusion of identities only. Whether one assumption entails another, and whether an implementation's assumptions are no stronger than the specification's, is the relation `ix://agent-ix/quire-specification/FR-177` defines and QSL's refinement backend decides. This requirement stops a result from being attributed to a request it was not established under.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-204-AC-1 | Given a stand-in result retaining exactly the request's assumption identities, the disposition is carried unchanged. | Test (TC-204) |
| FR-204-AC-2 | Given a stand-in result that proves refinement and retains one assumption identity absent from the request, the disposition is refused with `identity-mismatch` naming that identity, and carries no success value. | Test (TC-204) |
| FR-204-AC-3 | Given a stand-in result retaining a strict subset of the request's assumption identities, the disposition is carried unchanged. | Test (TC-204) |
| FR-204-AC-4 | Given a stand-in result that reports a violation and retains an extra assumption identity, the disposition is refused with the same cause and the counterexample is not presented as a carried result of the request. | Test (TC-204) |
| FR-204-AC-5 | Given the refused refinement beside a conformance claim and a composition claim, their dispositions equal those returned without the refinement claim, and no other backend is invoked for the refused claim. | Test (TC-204) |
| FR-204-AC-6 | Given a stand-in that itself returns the `protocol-assumption-not-established` cause with a strengthened-assumption witness, the cause and witness appear unchanged and the disposition is not refused. | Test (TC-204) |
| FR-204-AC-7 | Given a stand-in result that omits its retained assumption identities, and another that retains an empty set while the request holds assumptions, the first is refused with `identity-mismatch` naming the absent field and the second is carried as a strict subset. | Test (TC-204) |

## Dependencies

- [FR-301](./FR-301-admit-protocol-refinement-dispatch.md) supplies the refinement dispatch and the request this requirement checks the result against.
- [FR-202](./FR-202-carry-composition-and-refinement-results-unchanged.md) supplies the carrying that applies when the result is admitted or is the backend's own finding.
- [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) supplies the refusal of a disposition whose assumption component is substituted, and the no-fallback rule.
- [FR-029](./FR-029-return-one-disposition-per-requested-claim.md) and [FR-040](./FR-040-emit-no-success-beside-fatal-refusal.md) supply independence and the no-success-beside-refusal rule.
- [FR-008](./FR-008-return-typed-refusals.md) supplies the closed cause vocabulary that holds `identity-mismatch`.
- `ix://agent-ix/quire-specification/FR-177` owns the assumption-weakening relation; QSL FR-148 owns the backend's strengthened-assumption finding.
