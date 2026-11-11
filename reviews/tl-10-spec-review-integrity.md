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
