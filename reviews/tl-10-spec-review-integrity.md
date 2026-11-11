---
id: SR-2127026
title: "TL-10 FR-150..FR-153: integrity"
type: SpecReview
analysis: integrity
review_set: subset
scope: "agent-ix/quire-protocol; TL-10; PR #145 head 74ed09faaa3852ad3c4cd956fc24e2f3950e434b; spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md; spec/functional/FR-151-bind-communication-and-visibility-premises.md; spec/functional/FR-152-carry-projection-and-realizability-witness.md; spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/test-cases/TC-160-dispatch-projection-and-realizability-to-qsl.md; spec/test-cases/TC-161-bind-communication-and-visibility-premises.md; spec/test-cases/TC-162-carry-projection-and-realizability-witness.md; spec/test-cases/TC-163-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-150, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-152, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-153, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-160, type: reviews }
---
## Summary

Ticket: TL-10. integrity over PR #145 (spec-only; diff origin/main...HEAD at 74ed09f: FR-150..FR-153, TC-160..TC-163, spec/tests.md). `quire validate --scope .` passes the eight new artifacts; spec/tests.md fails only on the `Coverage Status` vs `Status` header assertion, which origin/main fails too. Upstream checked at quire-specification origin/main (FR-060, FR-216, FR-217, FR-241, FR-242, FR-243, FR-290, FR-292, FR-351) and quire-spec-language origin/main (ADR-014, ADR-020, ADR-023 vocabulary).

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) declares `review_policy.mode: require` but is `status: proposed`, so it is not an active profile and no enforced ix-flow run is claimed. Direct analyses run: base, integrity, ears-conformance, scope-boundary, dependency, failure-domain, evidence.

## Examined

- FR-150 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-150-AC-1 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-150-AC-3 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-150-AC-6 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-152-AC-5 (spec/functional/FR-152-carry-projection-and-realizability-witness.md)
- FR-153-AC-6 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- TC-160 (spec/test-cases/TC-160-dispatch-projection-and-realizability-to-qsl.md)

## Verdict

PASS WITH FINDINGS. The PR is consistent with merged FR-057, FR-058 and FR-301-AC-6. It uses no parallel vocabulary, keeps FR-302 to realizability only, and carries local-projection as a projected artifact with FR-030 correspondence. The unsupported dispositions map to truth and basis `unavailable` exactly as QSpec FR-241/242/243 derive. The findings below are defects only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-150 takes 'the set of QSL analysis backends the request may select from' as an input and every AC dispatches to a stand-in backend, but merged FR-005 says the dispatcher 'SHALL invoke only a reviewed built-in backend selected by the closed capability table ... a test double or caller callback is not a capability' (FR-005-AC-4). FR-150 also says it adds nothing beyond FR-005's closed dispatch. State how a stand-in enters the closed table (a test-only registration of the table, not a request input), or the ACs exercise the caller-supplied capability path FR-005 forbids. | spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md:23 |
| FND-002 | medium | FR-150 Behavior 'select the backend by the claim kind and the advertised backend capability (FR-292) and by nothing else' restates candidate selection that QSpec FR-290's candidate table owns (implemented in QSL's route). That table also selects by named backend and requirements and settles >1 candidate with no named backend as invalid-request ambiguous-backend and an unregistered named backend as unknown-backend. TC-160 registers 'one serving both kinds, one serving only each kind' together, which FR-290 settles ambiguous-backend unless the request names one. Cite the FR-290 candidate table instead of a simplified local rule, and give TC-160's fixtures a named backend or one candidate per kind. | spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md:23 |
| FND-003 | low | FR-152-AC-5's second clause (an artifact whose identity disagrees with the one derived from its selected tuple is refused under FR-030) restates FR-030-AC-5 nearly word for word. Cite FR-030-AC-5 rather than holding a second copy that can drift. | spec/functional/FR-152-carry-projection-and-realizability-witness.md:46 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | FR-150 Inputs say the stand-in is installed 'at the FR-331 provider boundary, exactly as TC-147 installs its refinement stand-in', but TC-147 says only 'using a test QSL refinement backend' and names no registry snapshot, FR-331 boundary or install seam. The cross-reference claims a precedent that does not exist. Drop the TC-147 clause and let FR-150's own sentence define the seam, or align TC-147 in a separate change. | spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md:30 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified at PR head ca84709. |
| FND-002 | fixed | Verified at PR head ca84709. |
| FND-003 | fixed | Verified at PR head ca84709. |
| FND-004 | fixed | Verified at PR head df47610 (round 2). |
| FND-005 | fixed | Round 1, 72743ff: Behavioural totals across these 61 rows: 353 acceptance criteria. 119 carry a living traced test, 106 belong to retired rows (their tests were deleted in QSL-1549 stage 1), 0 are blocked and 128 are planned or verify by Inspection or Analysis only. 119 + 106 + 0 + 128 = 353. |
| FND-006 | fixed | Round 1, 72743ff: When a `local-projection` or `realizability` claim is dispatched, the dispatcher SHALL bind the communication and visibility premises of the claim's fragment to the disposition and hand them to QSL's backend, taking each from the checked package where QSL's read view publishes it. |
| FND-007 | fixed | Round 2, 746d742: - The dispatcher shall give a disposition whose QSL verdict is `Inconclusive` the truth and settlement basis the backend states, and neither truth `satisfied` nor truth `violated`. |

## New findings (quire-protocol#148 spec delta)

Spec delta of quire-protocol#148 at 9a7c29cbccf869b7df1aa109a0b541442416656c (`git diff origin/main...HEAD -- spec/`): FR-151 (AC-1 narrowed, AC-7 appended), FR-152 (AC-5 narrowed, AC-6 appended), TC-162 procedure, spec/tests.md rows FR-150..FR-153 and TC-160..TC-163.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | low | The rows change criterion counts (FR-151 6->7, FR-152 5->6) and move 27 criteria from planned to living-traced. The Overview ('474 acceptance criteria'), the Behavioural totals line ('347 ... 88 carry a living traced test ... 153 are planned') and the Owed acceptance criteria table were not recomputed. | spec/tests.md:228 |
| FND-006 | medium | FR-151's Description, Inputs and Behavior bullet 1 still say the dispatcher hands the backend all five channel-premise components and the role internality 'exactly as the checked package authors them'. AC-1 was narrowed and AC-7 split off because no read view provides them. The FR statement now promises what no criterion verifies, and only spec/tests.md records the gap. Say in FR-151 that this dependency is outstanding. | spec/functional/FR-151-bind-communication-and-visibility-premises.md:55 |

## New findings (quire-protocol#148 disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | low | The new FR-151 bullet forces truth `unavailable` for an `Inconclusive` verdict 'because FR-242 reserves `pending` for an open decision scope with settlement basis `unsettled`, which an `Inconclusive` verdict does not state'. FR-242's own `pending` row is completed execution with basis `unsettled`, and it projects to E02 logical.outcome `inconclusive`. The TC-161 stand-in returns basis `Unsettled`, yet the dispatcher overrides it to `unavailable`. Either drop the stated rationale and carry the backend's own `pending`/`unavailable` truth (clamping only `satisfied`/`violated`), or cite the QSpec line that maps V-6 `Inconclusive` to `unavailable`. Not blocking. | spec/functional/FR-151-bind-communication-and-visibility-premises.md:Behavior |
