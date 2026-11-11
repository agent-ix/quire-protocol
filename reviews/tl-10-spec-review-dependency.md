---
id: SR-2127029
title: "TL-10 FR-150..FR-153: dependency"
type: SpecReview
analysis: dependency
review_set: subset
scope: "agent-ix/quire-protocol; TL-10; PR #145 head 74ed09faaa3852ad3c4cd956fc24e2f3950e434b; spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md; spec/functional/FR-151-bind-communication-and-visibility-premises.md; spec/functional/FR-152-carry-projection-and-realizability-witness.md; spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/test-cases/TC-160-dispatch-projection-and-realizability-to-qsl.md; spec/test-cases/TC-161-bind-communication-and-visibility-premises.md; spec/test-cases/TC-162-carry-projection-and-realizability-witness.md; spec/test-cases/TC-163-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-150, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-151, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-152, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-153, type: reviews }
---
## Summary

Ticket: TL-10. dependency over PR #145 (spec-only; diff origin/main...HEAD at 74ed09f: FR-150..FR-153, TC-160..TC-163, spec/tests.md). `quire validate --scope .` passes the eight new artifacts; spec/tests.md fails only on the `Coverage Status` vs `Status` header assertion, which origin/main fails too. Upstream checked at quire-specification origin/main (FR-060, FR-216, FR-217, FR-241, FR-242, FR-243, FR-290, FR-292, FR-351) and quire-spec-language origin/main (ADR-014, ADR-020, ADR-023 vocabulary).

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) declares `review_policy.mode: require` but is `status: proposed`, so it is not an active profile and no enforced ix-flow run is claimed. Direct analyses run: base, integrity, ears-conformance, scope-boundary, dependency, failure-domain, evidence.

## Examined

- FR-150 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-151 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)
- FR-152 (spec/functional/FR-152-carry-projection-and-realizability-witness.md)
- FR-153 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)

## Verdict

PASS WITH FINDINGS. The PR is consistent with merged FR-057, FR-058 and FR-301-AC-6. It uses no parallel vocabulary, keeps FR-302 to realizability only, and carries local-projection as a projected artifact with FR-030 correspondence. The unsupported dispositions map to truth and basis `unavailable` exactly as QSpec FR-241/242/243 derive. The findings below are defects only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-153's Outputs bind the unsupported disposition 'to the request's exact premises (FR-030)' and FR-153-AC-7 tests that retention, but FR-153's relationships carry no FR-030 edge (FR-150, FR-151 and FR-152 all carry one). Add `FR-030 depends_on`. | spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md:22 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified at PR head ca84709. |
| FND-002 | fixed | Round 1, 72743ff: / FR-152 / FR-152-AC-1..AC-7 / TC-162 (AC-1..AC-5, AC-7); no living test for AC-6 / 🚧 Partial — 6 of 7 criteria have living traced tests; AC-6 awaits quire-specification's FR-249 derived-artifact identity rule and FR-062 `resultId` rule (`ix://agent-ix/quire-specification/FR-249`, `ix://agent-ix/quire-specification/FR-062`). / |

## New findings (quire-protocol#148 spec delta)

Spec delta of quire-protocol#148 at 9a7c29cbccf869b7df1aa109a0b541442416656c (`git diff origin/main...HEAD -- spec/`): FR-151 (AC-1 narrowed, AC-7 appended), FR-152 (AC-5 narrowed, AC-6 appended), TC-162 procedure, spec/tests.md rows FR-150..FR-153 and TC-160..TC-163.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | The tests.md FR-152 row says AC-6 'awaits the FR-249 derived-artifact identity rule'. In this repository ADR-010 reserves FR-200..FR-249 for TL-9 composition, so a bare 'FR-249' reads as a local requirement. Qualify it as ix://agent-ix/quire-specification/FR-249, with FR-062, as FR-030 cites them. | spec/tests.md:220 |
