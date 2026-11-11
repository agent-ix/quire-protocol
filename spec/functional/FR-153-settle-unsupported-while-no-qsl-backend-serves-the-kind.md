---
id: FR-153
title: "Settle projection and realizability claims unsupported while no QSL backend serves the kind"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-004, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-029, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-030, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-057, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-150, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-301, type: references }
  - { target: ix://agent-ix/quire-specification/FR-060, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-241, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-290, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-242, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-243, type: depends_on }
---
# FR-153: Settle projection and realizability claims unsupported while no QSL backend serves the kind

## Description

When a `local-projection` or `realizability` claim is requested and no accepted QSL backend serves that claim kind, the dispatcher SHALL settle the claim `unsupported`, leaving the independent finite global conformance result preserved.

## Inputs

One requested `local-projection` or `realizability` claim, the set of QSL analysis backends the request may select from, and the independently requested finite global conformance claim with its result.

## Outputs

One `unsupported` disposition (FR-241) per such claim, carrying QSL's cause `unsupported_projection`/`unsupported-requested-capability` and a warning naming the required claim kind and the named backend when the request names one, bound to the request's exact premises (FR-030); the global conformance result unchanged.

## Behavior

- When the claim's FR-290 candidate set is empty, the dispatcher shall settle the claim with assessment execution `unsupported`.
- The dispatcher shall carry the cause `unsupported_projection`/`unsupported-requested-capability` as FR-057 states and define no second name for it.
- The dispatcher shall give the unsupported disposition the truth `unavailable` and the settlement basis `unavailable`, which `ix://agent-ix/quire-specification/FR-242` and `ix://agent-ix/quire-specification/FR-243` derive from it.
- The dispatcher shall name the required claim kind, and the named backend when the request names one, in a warning on the disposition.
- The dispatcher shall return the disposition as a result and not as a hold, a blocker or a refusal error.
- The dispatcher shall derive no projection or realizability result from a satisfiable finite trace or from a conformant global result (`ix://agent-ix/quire-specification/FR-060`).
- The dispatcher shall substitute no local approximation and no other backend for the claim.
- The dispatcher shall leave the finite global conformance result and every other requested claim's disposition unchanged (FR-029).
- When a backend serving the claim kind is registered, the dispatcher shall dispatch an identical request to it under FR-150 with no change to the request.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-153-AC-1 | Given an empty registry and a request for `local-projection`, `realizability` and `global-conformance`, the first two dispositions have assessment execution `unsupported` and cause `unsupported_projection`/`unsupported-requested-capability`, and the third equals the one returned when the first two are not requested. | Test (TC-163) |
| FR-153-AC-2 | Given the same request, each unsupported disposition holds truth `unavailable` and settlement basis `unavailable`, and neither holds truth `satisfied` or `violated`. | Test (TC-163) |
| FR-153-AC-3 | Given a satisfiable finite trace with a conformant global result and an empty registry, no `local-projection` or `realizability` result is reported as holding. | Test (TC-163) |
| FR-153-AC-4 | Given an empty registry, the dispatch returns the dispositions as a result with no error, hold, blocker or refusal, each warning names its required claim kind, and a request naming a backend adds that backend to the warning. | Test (TC-163) |
| FR-153-AC-5 | Given a stand-in that serves only `refinement`, a `local-projection` claim and a `realizability` claim settle `unsupported` and the stand-in records no call for them. | Test (TC-163) |
| FR-153-AC-6 | Given the request of FR-153-AC-1 run again byte-identically after a stand-in serving `local-projection` is registered, the `local-projection` disposition equals the stand-in's result and the `realizability` disposition is still `unsupported`. | Test (TC-163) |
| FR-153-AC-7 | Given an empty registry, each unsupported disposition retains the request's exact fragment, backend, assumptions, premises and source identities equal to the request's, and its result, witness and projected-artifact fields are absent. | Test (TC-163) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Assessment-execution disposition (`unsupported`) | `ix://agent-ix/quire-specification/FR-241` | value |
| Truth disposition (`unavailable`) | `ix://agent-ix/quire-specification/FR-242` | value |
| Settlement basis (`unavailable`) | `ix://agent-ix/quire-specification/FR-243` | value |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) records that QSL provides claim vocabulary and routing for the two kinds and no decision procedure.
- [FR-004](./FR-004-assess-finite-global-conformance.md) supplies the finite global conformance result that stays available.
- [FR-029](./FR-029-return-one-disposition-per-requested-claim.md) supplies the independence of dispositions.
- [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) supplies the premise binding the unsupported disposition retains.
- `ix://agent-ix/quire-specification/FR-290` supplies the candidate table whose empty row settles `unsupported`.
- [FR-057](./FR-057-map-qsl-unsupported-outcomes-to-typed-dispositions.md) supplies the cause-carrying pattern and the `unsupported-requested-capability` name.
- [FR-150](./FR-150-dispatch-projection-and-realizability-to-qsl.md) supplies the dispatch used once a backend serves the kind.
- [FR-301](./FR-301-admit-protocol-refinement-dispatch.md) states the same absence pattern for the `refinement` claim kind.
- `ix://agent-ix/quire-specification/FR-060` states that an unsupported advanced claim does not erase a valid independent global result.
