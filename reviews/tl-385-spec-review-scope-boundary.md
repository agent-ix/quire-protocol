---
id: SR-2127015
title: "TL-385 ADR-010 protocol/QSL split: scope-boundary"
type: SpecReview
analysis: scope-boundary
review_set: subset
scope: "agent-ix/quire-protocol; TL-385; PR #141 head 2e751a81781f6b5ddc96ac578289db0372908f1c; spec/adr/ADR-010-consume-qsl-protocol-analysis-own-assessment-and-dispatch.md; spec/functional/FR-301-admit-protocol-refinement-dispatch.md; spec/assurance/AD-001-protocol-library-boundaries.md; spec/spec.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/ADR-010, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-301, type: reviews }
---
## Summary

Ticket: TL-385. Scope-boundary review of the library/QSL split, with every QSL
claim in ADR-010 checked against quire-spec-language origin/main 4ffe6b7 by
`git show`/`git grep` (the local QSL checkout is stale and was not read).

## Assurance Context

AP-001 is proposed, not active; direct analysis, no enforced run claimed.

## Verdict

**PASS WITH FINDINGS.** Verified against QSL main: all cited files exist;
`roles.rs` implements spawn/retire/until/max_live_instances; `steps.rs` has
all/any/quorum(n) and outstanding cancel; `program.rs`/`settle.rs` implement
`repeat` with PS-6 progress; `model_check/check.rs` has the RequiresBound
pre-check; `check_refinement` is in `qsl-analyze/src/model_check/refine.rs` and
ADR-020 RE-1 is the refinement product; QSL FR-142..FR-149 and FR-530 exist;
`planner/compose.rs` discharges rely/liveness-dependency slots and computes no
fixed point (G6 confirmed). `Capability::LocalProjection`/`Realizability` appear
only in vocabulary, gating, routing and the test
`protocol_kinds_apply_only_to_protocol_claims`; no projection or realizability
engine exists and an unserved request settles
`unsupported_projection/unsupported-requested-capability` (G1, G2 confirmed).
`Unsupported::JoinPredicate`, `UnruledOutstanding`, `SymbolicCapacity`,
`ActivationOnEachTemplate`, `Capture`, `UnorderedLossSelector` and
`UnenumerableRecord` exist (G3, G4, G7 confirmed). `invariant`/`variant` are only
parsed and lowered (`qsl-forms`, `controls.rs`); no discharge engine found (G5
confirmed by the same negative search). The library-side claims hold: `src/` has
no projection, realizability, refinement or quorum code (the projection hits are
E02 result projection). Per-ticket ownership covers TL-6, TL-8, TL-9, TL-10,
TL-11 and TL-12 as TL-385 AC2 requires. One medium boundary ambiguity remains in
FR-301's Inputs.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-301 Inputs still lists "the FR-002 linked participant/role/channel graph for the paired implementation and specification protocols" as the claim's input, as when the library built the product; after the amendment it is unstated what the library hands QSL's refinement backend (the checked package's protocol subjects, as `check_refinement` takes, or the linked graph), so two implementers could build different dispatch inputs | spec/functional/FR-301-admit-protocol-refinement-dispatch.md:44-47 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | ec54159 states the library hands QSL the checked-package references (FR-043, FR-044) as the request item's subjects and uses the FR-002 graph only to bind the returned result |
