---
id: FR-150
title: "Dispatch local-projection and realizability claims to QSL's backend"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-005, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-029, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-030, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-301, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-153, type: references }
  - { target: ix://agent-ix/quire-specification/FR-060, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-241, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-290, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-292, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/ADR-017, type: depends_on }
---
# FR-150: Dispatch local-projection and realizability claims to QSL's backend

## Description

When a `local-projection` or `realizability` claim is requested and an accepted QSL analysis backend serves that claim kind, the dispatcher SHALL send the claim to that backend and carry its result unchanged.

## Inputs

One requested `local-projection` or `realizability` claim with its admitted fragment, optional named backend, assumptions and source identities; the FR-002 linked graph, used only to bind the returned result to source identities; and the QSL registry snapshot, from which `ix://agent-ix/quire-specification/FR-290` computes the claim's candidate set.

The registry snapshot is QSL's, not a capability table of this library. A test installs its stand-in backend as a registration in that snapshot at the FR-331 provider boundary. The dispatcher receives no caller callback and the closed built-in table of FR-005 gains no entry for it.

## Outputs

One disposition per requested claim (FR-029), carrying the backend's result, refusal or `unsupported` cause in QSL's own vocabulary, bound to the exact fragment, backend, assumptions and source identities the claim was requested against (FR-030).

## Behavior

- The dispatcher shall take the candidate set of the claim from QSL's registry snapshot under `ix://agent-ix/quire-specification/FR-290`.
- When FR-290 settles the claim with exactly one candidate, the dispatcher shall send the claim to that candidate's backend.
- The dispatcher shall not select among several candidates.
- The dispatcher shall carry an FR-290 settlement of the candidate table (`ambiguous-backend`, `unknown-backend`, or an empty candidate set per FR-153) unchanged and send the claim to no backend.
- The dispatcher shall carry the backend's result, refusal or `unsupported` cause unchanged and in the backend's own vocabulary, including `unsupported_projection`/`tool-unavailable`.
- The dispatcher shall compute no projection, no realizability verdict, no strategy and no witness from the FR-002 linked graph.
- The dispatcher shall derive no verdict for a claim whose backend returned none.
- The dispatcher shall return `failed` (FR-241) for exactly the claim whose selected backend failed.
- The dispatcher shall invoke no other backend for a claim whose selected backend failed.
- The dispatcher shall leave the finite global conformance disposition and every other requested claim's disposition unchanged (FR-029).
- The dispatcher shall record on the disposition the backend it selected the claim against.
- The provider boundary shall refuse a second provider for a backend identity that already has one.
- The dispatcher shall introduce no dispatch path, resource guard or fallback backend beyond FR-005's closed dispatch.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-150-AC-1 | Given a registered stand-in serving `local-projection` that returns a sentinel result, a requested `local-projection` claim is sent to it exactly once with the claim's admitted fragment, assumptions and source identities, and the disposition carries the sentinel result byte-for-byte. | Test (TC-160) |
| FR-150-AC-2 | Given a registered stand-in serving `realizability` that returns a sentinel result, a requested `realizability` claim is sent to it exactly once and the disposition carries the sentinel result byte-for-byte. | Test (TC-160) |
| FR-150-AC-3 | Given a stand-in serving only `realizability`, a `local-projection` claim is sent to no backend, and a stand-in serving only `local-projection` is sent no `realizability` claim. | Test (TC-160) |
| FR-150-AC-4 | Given a stand-in that returns one fixed sentinel result for every request, a conformant fragment and a non-conformant fragment both yield a disposition carrying that sentinel unchanged, and given a stand-in that returns no verdict the disposition carries none. | Test (TC-160) |
| FR-150-AC-5 | Given a stand-in that returns a refusal with a distinct cause, one that returns an `unsupported` cause, and one that returns `unsupported_projection`/`tool-unavailable`, the disposition carries each cause name unchanged in the backend's vocabulary and no other candidate is invoked. | Test (TC-160) |
| FR-150-AC-6 | Given a stand-in that fails for one of three requested claims, that claim's disposition is `failed`, no second backend is invoked for it, and the other two dispositions equal those returned when the backend does not fail. | Test (TC-160) |
| FR-150-AC-7 | Given a request for `local-projection`, `realizability` and `global-conformance` together, three dispositions are returned, and the `global-conformance` disposition equals the one returned when the other two claims are not requested. | Test (TC-160) |
| FR-150-AC-8 | Given two registered stand-ins that both serve `realizability` and a request naming neither, the claim settles under FR-290 as `ambiguous-backend` naming both candidates in candidate order and neither stand-in records a call, and given a request naming an unregistered backend the claim settles `unknown-backend`. | Test (TC-160) |
| FR-150-AC-9 | Given the same two stand-ins, a request naming one of them sends the claim to that stand-in only, and the result is identical under every registration order. | Test (TC-160) |
| FR-150-AC-10 | Given a request naming no backend and one stand-in serving the claim kind, the disposition records that stand-in as the backend selected, and a disposition settled without a selected backend records none. | Test (TC-160) |
| FR-150-AC-11 | Given a second provider registered for a backend identity that already has one, the registration is refused and the first provider is the one called. | Test (TC-160) |

## Adopted vocabulary

| Noun | Upstream object | Identity kind |
| --- | --- | --- |
| Protocol claim kind (`local-projection`, `realizability`) | `ix://agent-ix/quire-specification/FR-290` | value |
| Analysis backend | `ix://agent-ix/quire-specification/FR-292` | authored |
| Candidate table and backend selection | `ix://agent-ix/quire-specification/FR-290` | value |
| Assessment-execution disposition (`failed`) | `ix://agent-ix/quire-specification/FR-241` | value |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes that this library dispatches the two claim kinds and implements neither decision procedure.
- `ix://agent-ix/quire-specification/FR-290` supplies the candidate table and its settlements, which this requirement carries and does not restate.
- [FR-002](./FR-002-link-finite-protocol-representation.md) supplies the linked graph whose source identities bind the returned result.
- [FR-005](./FR-005-dispatch-independent-protocol-claims.md), [FR-029](./FR-029-return-one-disposition-per-requested-claim.md) and [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) supply closed dispatch, independence and premise binding.
- [FR-153](./FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md) states the empty-candidate settlement.
- [FR-301](./FR-301-admit-protocol-refinement-dispatch.md) states the same dispatch-and-carry rule for the `refinement` claim kind.
- `ix://agent-ix/quire-specification/FR-060` states the independence of global conformance from the advanced claims.
- QSL ADR-017 fixes the protocol boundary of the checked package this library reads.
