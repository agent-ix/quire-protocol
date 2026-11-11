---
id: FR-200
title: "Dispatch composition claims to QSL's composition backend"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-005, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-029, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-030, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-043, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-044, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-301, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-176, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-290, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-292, type: depends_on }
  - { target: ix://agent-ix/quire-specification/FR-489, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-530, type: depends_on }
---
# FR-200: Dispatch composition claims to QSL's composition backend

## Description

When a `composition` claim is requested and an accepted FR-292 analysis backend serves it, the dispatcher SHALL send the claim to QSL's composition backend and carry that backend's result. The library SHALL build no composition graph, discharge no guarantee and settle no composition locally.

## Inputs

One requested `composition` claim (an `ix://agent-ix/quire-specification/FR-290` label) with its admitted fragment, selected backend, assumptions and source identities. The checked-package references (FR-043, FR-044) naming the component protocols, which this library hands QSL's composition backend as that request item's subjects. The composition rule and the four premise lists of `ix://agent-ix/quire-specification/FR-489`. The FR-002 linked graph, used only to bind the returned result to source identities.

## Outputs

One source-bound composition disposition retaining its exact admitted fragment, backend, assumptions and source and derived-artifact correspondence as FR-030 requires of every disposition, or one explicit `unsupported` disposition when no accepted backend serves the claim.

## Behavior

- When an accepted backend serves a requested `composition` claim, the dispatcher shall send exactly one composition request item for that claim to QSL's composition backend.
- The dispatcher shall carry the backend's result in the composition disposition (FR-202 states what carrying means).
- When no accepted FR-292 analysis backend serves a `composition` claim, the dispatcher shall settle that claim `unsupported` with a warning naming the required capability.
- When a composition claim settles `unsupported`, the dispatcher shall leave the independent global conformance result and every other claim's disposition unchanged (FR-029).
- The dispatcher shall not treat the `unsupported` settlement as a hold, a blocker or a refusal error.
- The dispatcher shall not carry the complete-global-conformance premise set as evidence for a composition disposition's own success, as `ix://agent-ix/quire-specification/FR-290` requires of every non-global claim.
- The dispatcher shall not discharge a premise, order guarantees or evaluate a dependency graph, because QSL's composition planner owns the discharge (QSL FR-530, `ix://agent-ix/quire-specification/FR-176`).

This requirement extends FR-005's closed dispatch and FR-029 and FR-030's independence and premise binding to the `composition` claim kind. FR-301 states the same dispatch for `refinement`. This requirement introduces no separate dispatch path, resource guard or fallback backend.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-200-AC-1 | Given a requested `composition` claim with an accepted composition backend, a recording stand-in for the QSL call boundary receives exactly one composition request item whose subjects are the component packages' references, and the disposition carries the stand-in's returned result. | Test (TC-200) |
| FR-200-AC-2 | Given a `composition` claim with no accepted FR-292 backend, the disposition is `unsupported` with a warning naming the required capability, even when every premise in the request is supplied by a matching component guarantee, and the settlement is not a hold, blocker or refusal error. | Test (TC-200) |
| FR-200-AC-3 | Given an `unsupported` composition claim requested beside finite global conformance and a refinement claim, the conformance disposition and the refinement disposition equal those returned without the composition claim. | Test (TC-200) |
| FR-200-AC-4 | A composition disposition never carries the complete-global-conformance premise set as evidence for its own success, and a successful global conformance result does not change a composition disposition. | Test (TC-200) |
| FR-200-AC-5 | Removing or substituting the composition disposition's admitted fragment, backend, assumption identities or source identity one at a time refuses only that disposition (FR-030) and leaves every independent disposition unchanged. | Test (TC-200) |
| FR-200-AC-6 | Given a stand-in that returns a result different from what the request's own premises would imply if evaluated locally, the disposition equals the stand-in's result, and no result appears when the stand-in returns none. | Test (TC-200) |

## Dependencies

- [ADR-010](../adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md) fixes that composition is dispatched and carried here and decided in QSL.
- [FR-301](./FR-301-admit-protocol-refinement-dispatch.md) states the dispatch pattern this requirement mirrors for the `composition` kind.
- [FR-005](./FR-005-dispatch-independent-protocol-claims.md), [FR-029](./FR-029-return-one-disposition-per-requested-claim.md) and [FR-030](./FR-030-bind-each-disposition-to-its-exact-premises.md) supply closed dispatch, independence and premise binding.
- [FR-043](./FR-043-admit-the-checked-package-handoff.md) and [FR-044](./FR-044-link-from-a-typed-read-view-of-the-checked-package.md) supply the checked-package references handed to the backend, and [FR-002](./FR-002-link-finite-protocol-representation.md) the linked graph.
- `ix://agent-ix/quire-specification/FR-176`, `ix://agent-ix/quire-specification/FR-489` and QSL FR-530 own the composition contract, the four premise lists and the discharge this requirement does not restate.
- `ix://agent-ix/quire-specification/FR-290` and `ix://agent-ix/quire-specification/FR-292` supply the claim kind and the analysis backend.
