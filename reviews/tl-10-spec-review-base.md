---
id: SR-2127025
title: "TL-10 FR-150..FR-153: base"
type: SpecReview
analysis: base
review_set: subset
scope: "agent-ix/quire-protocol; TL-10; PR #145 head 74ed09faaa3852ad3c4cd956fc24e2f3950e434b; spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md; spec/functional/FR-151-bind-communication-and-visibility-premises.md; spec/functional/FR-152-carry-projection-and-realizability-witness.md; spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/test-cases/TC-160-dispatch-projection-and-realizability-to-qsl.md; spec/test-cases/TC-161-bind-communication-and-visibility-premises.md; spec/test-cases/TC-162-carry-projection-and-realizability-witness.md; spec/test-cases/TC-163-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-150, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-151, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-152, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-153, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-160, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-161, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-162, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TC-163, type: reviews }
---
## Summary

Ticket: TL-10. base over PR #145 (spec-only; diff origin/main...HEAD at 74ed09f: FR-150..FR-153, TC-160..TC-163, spec/tests.md). `quire validate --scope .` passes the eight new artifacts; spec/tests.md fails only on the `Coverage Status` vs `Status` header assertion, which origin/main fails too. Upstream checked at quire-specification origin/main (FR-060, FR-216, FR-217, FR-241, FR-242, FR-243, FR-290, FR-292, FR-351) and quire-spec-language origin/main (ADR-014, ADR-020, ADR-023 vocabulary).

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) declares `review_policy.mode: require` but is `status: proposed`, so it is not an active profile and no enforced ix-flow run is claimed. Direct analyses run: base, integrity, ears-conformance, scope-boundary, dependency, failure-domain, evidence.

## Examined

- FR-150 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-150-AC-1 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-150-AC-2 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-150-AC-3 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-150-AC-4 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-150-AC-5 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-150-AC-6 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-150-AC-7 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-151 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)
- FR-151-AC-1 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)
- FR-151-AC-2 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)
- FR-151-AC-3 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)
- FR-151-AC-4 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)
- FR-151-AC-5 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)
- FR-151-AC-6 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)
- FR-152 (spec/functional/FR-152-carry-projection-and-realizability-witness.md)
- FR-152-AC-1 (spec/functional/FR-152-carry-projection-and-realizability-witness.md)
- FR-152-AC-2 (spec/functional/FR-152-carry-projection-and-realizability-witness.md)
- FR-152-AC-3 (spec/functional/FR-152-carry-projection-and-realizability-witness.md)
- FR-152-AC-4 (spec/functional/FR-152-carry-projection-and-realizability-witness.md)
- FR-152-AC-5 (spec/functional/FR-152-carry-projection-and-realizability-witness.md)
- FR-153 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- FR-153-AC-1 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- FR-153-AC-2 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- FR-153-AC-3 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- FR-153-AC-4 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- FR-153-AC-5 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- FR-153-AC-6 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- FR-153-AC-7 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- TC-160 (spec/test-cases/TC-160-dispatch-projection-and-realizability-to-qsl.md)
- TC-161 (spec/test-cases/TC-161-bind-communication-and-visibility-premises.md)
- TC-162 (spec/test-cases/TC-162-carry-projection-and-realizability-witness.md)
- TC-163 (spec/test-cases/TC-163-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- spec/tests.md (spec/tests.md)

## Verdict

PASS WITH FINDINGS. The PR is consistent with merged FR-057, FR-058 and FR-301-AC-6. It uses no parallel vocabulary, keeps FR-302 to realizability only, and carries local-projection as a projected artifact with FR-030 correspondence. The unsupported dispositions map to truth and basis `unavailable` exactly as QSpec FR-241/242/243 derive. The findings below are defects only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | spec/tests.md conflicts with main e847803 (TL-6 #144 merged after this branch's base 9986cd8), and the PR leaves the prose totals (49 FRs, 24 TCs, 348 ACs) and the Test Execution Summary rows (40 FR rows, 24 TC rows) unchanged although it adds 4 FRs, 4 TCs and 25 ACs. The behavioural line is right against its base (50+4=54 rows, 275+25=300 ACs, 81+25=106 planned) but is stale on main. Rebased on e847803 the figures are: 56 rows, 313 ACs, 119 planned; prose 65 FRs, 35 TCs, 428 ACs; Summary FR rows 56 (9/0/0/28/19) and TC rows 35 (9/0/0/18/8). | spec/tests.md:217 |
