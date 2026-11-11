---
id: SR-2127031
title: "TL-10 FR-150..FR-153: evidence"
type: SpecReview
analysis: evidence
review_set: subset
scope: "agent-ix/quire-protocol; TL-10; PR #145 head 74ed09faaa3852ad3c4cd956fc24e2f3950e434b; spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md; spec/functional/FR-151-bind-communication-and-visibility-premises.md; spec/functional/FR-152-carry-projection-and-realizability-witness.md; spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/test-cases/TC-160-dispatch-projection-and-realizability-to-qsl.md; spec/test-cases/TC-161-bind-communication-and-visibility-premises.md; spec/test-cases/TC-162-carry-projection-and-realizability-witness.md; spec/test-cases/TC-163-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-150, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-153, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-160, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-161, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-162, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-163, type: reviews }
---
## Summary

Ticket: TL-10. evidence over PR #145 (spec-only; diff origin/main...HEAD at 74ed09f: FR-150..FR-153, TC-160..TC-163, spec/tests.md). `quire validate --scope .` passes the eight new artifacts; spec/tests.md fails only on the `Coverage Status` vs `Status` header assertion, which origin/main fails too. Upstream checked at quire-specification origin/main (FR-060, FR-216, FR-217, FR-241, FR-242, FR-243, FR-290, FR-292, FR-351) and quire-spec-language origin/main (ADR-014, ADR-020, ADR-023 vocabulary).

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) declares `review_policy.mode: require` but is `status: proposed`, so it is not an active profile and no enforced ix-flow run is claimed. Direct analyses run: base, integrity, ears-conformance, scope-boundary, dependency, failure-domain, evidence.

## Examined

- FR-150-AC-4 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-153-AC-6 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- FR-153-AC-7 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- TC-160 (spec/test-cases/TC-160-dispatch-projection-and-realizability-to-qsl.md)
- TC-161 (spec/test-cases/TC-161-bind-communication-and-visibility-premises.md)
- TC-162 (spec/test-cases/TC-162-carry-projection-and-realizability-witness.md)
- TC-163 (spec/test-cases/TC-163-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)

## Verdict

PASS WITH FINDINGS. The PR is consistent with merged FR-057, FR-058 and FR-301-AC-6. It uses no parallel vocabulary, keeps FR-302 to realizability only, and carries local-projection as a projected artifact with FR-030 correspondence. The unsupported dispositions map to truth and basis `unavailable` exactly as QSpec FR-241/242/243 derive. The findings below are defects only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-153-AC-6 ends with 'and the dispatch code path called is the same one', a code-structure claim that the dispatch result cannot show and only inspection could check. Earlier reviews in this wave flagged this AC pattern. State it as behaviour instead: with the stand-in added, the stand-in records exactly one call carrying the claim's premises, and the disposition equals its result. | spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md:53 |
| FND-002 | low | FR-150-AC-4's oracle, 'a verdict opposite to the one a construction over the FR-002 linked graph would give', needs the test to compute a projection or realizability verdict, which ADR-010 and FR-150 say this library never computes. Use a fixture with a hand-labelled expected verdict, or a sentinel verdict that does not depend on the graph. | spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md:52 |
| FND-003 | low | FR-153-AC-7 'no field of them holds a value the library computed for the claim' does not list the fields, so a test cannot tell a computed value from a carried constant (the kind, the cause name, `unavailable`). List the fields and the source each must equal. | spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md:54 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified at PR head ca84709. |
| FND-002 | fixed | Verified at PR head ca84709. |
| FND-003 | fixed | Verified at PR head ca84709. |
