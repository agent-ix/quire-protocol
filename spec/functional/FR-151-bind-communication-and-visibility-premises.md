---
id: FR-151
title: "Bind communication and visibility premises to projection and realizability dispositions"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-030, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-044, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-050, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-058, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-150, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-216, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-217, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-242, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-243, type: depends_on }
---
# FR-151: Bind communication and visibility premises to projection and realizability dispositions

## Description

When a `local-projection` or `realizability` claim is dispatched, the dispatcher SHALL bind the exact communication and visibility premises the checked package authors for the claim's fragment to the disposition and hand them to QSL's backend.

## Inputs

The channel premise (sender, receiver, message type, ordering policy and delivery policy) of every channel and the role internality of every role in the claim's admitted fragment, read from the checked package through FR-044; and the claim's requested premises.

## Outputs

The premises as the backend received them, retained on the disposition (FR-030), or one FR-030 refusal of the disposition naming the absent premise. A disposition whose premise is absent, or whose QSL outcome is non-conclusive, carries no truth `satisfied` or `violated`.

## Behavior

- The dispatcher shall hand the backend each channel premise and each role internality exactly as the checked package authors them.
- The dispatcher shall retain on the disposition each premise it handed the backend.
- When a channel premise or a role internality of the fragment is absent, the dispatcher shall refuse that disposition under FR-030 naming the absent premise.
- When a channel premise or a role internality of the fragment is absent, the dispatcher shall invoke no backend for that claim.
- The dispatcher shall supply no default, inferred or sampled premise.
- The dispatcher shall assume no full observability of an `external` role (`ix://agent-ix/quire-specification/FR-217`).
- When QSL reports an outcome that is not conclusive, including a requires-bound outcome (FR-058), the dispatcher shall carry it unchanged.
- The dispatcher shall give a disposition with an absent premise or a non-conclusive QSL outcome neither truth `satisfied` nor truth `violated`.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-151-AC-1 | Given a fragment with channels and roles, the stand-in backend's recorded call carries each channel premise (all five components) and each role internality equal to the checked package's, and the disposition retains the same premises. | Test (TC-161) |
| FR-151-AC-2 | Given two requests that differ in exactly one component of one channel premise, their request identities and their disposition provenance differ. | Test (TC-161) |
| FR-151-AC-3 | Given a request from which one channel premise or one role internality is removed, that disposition is refused under FR-030 naming the absent premise, the stand-in backend records no call for it, and every independent disposition is unchanged. | Test (TC-161) |
| FR-151-AC-4 | Given a fragment with an `external` role, the backend call carries `external` for it and no premise implying that the role is fully observable. | Test (TC-161) |
| FR-151-AC-5 | Given a stand-in backend that returns a requires-bound outcome and one that returns a non-conclusive result, each disposition carries the outcome unchanged and holds neither truth `satisfied` nor truth `violated`. | Test (TC-161) |
| FR-151-AC-6 | Given a request with a premise absent, the disposition holds neither truth `satisfied` nor truth `violated`, and no premise value in the recorded backend calls is absent from the checked package. | Test (TC-161) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Channel premise (five components) | `ix://agent-ix/quire-specification/FR-216` | value |
| Role internality | `ix://agent-ix/quire-specification/FR-217` | value |
| Truth disposition | `ix://agent-ix/quire-specification/FR-242` | value |
| Settlement basis | `ix://agent-ix/quire-specification/FR-243` | value |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes premise binding as library-owned dispatch work.
- [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) supplies the retention and refusal rule this requirement applies to the two claim kinds.
- [FR-044](./FR-044-link-from-a-typed-read-view-of-the-checked-package.md) supplies the read view the premises come from.
- [FR-050](./FR-050-link-role-declarations-without-minting-identity.md) supplies the role declarations whose internality is read.
- [FR-058](./FR-058-map-requires-bound-outcomes-to-typed-dispositions.md) supplies the requires-bound outcome carried here.
- [FR-150](./FR-150-dispatch-projection-and-realizability-to-qsl.md) supplies the dispatch this requirement binds premises to.
