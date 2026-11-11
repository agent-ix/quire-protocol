---
id: SR-2127028
title: "TL-10 FR-150..FR-153: scope-boundary"
type: SpecReview
analysis: scope-boundary
review_set: subset
scope: "agent-ix/quire-protocol; TL-10; PR #145 head 74ed09faaa3852ad3c4cd956fc24e2f3950e434b; spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md; spec/functional/FR-151-bind-communication-and-visibility-premises.md; spec/functional/FR-152-carry-projection-and-realizability-witness.md; spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/test-cases/TC-160-dispatch-projection-and-realizability-to-qsl.md; spec/test-cases/TC-161-bind-communication-and-visibility-premises.md; spec/test-cases/TC-162-carry-projection-and-realizability-witness.md; spec/test-cases/TC-163-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-151, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-152, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-153, type: reviews }
---
## Summary

Ticket: TL-10. scope-boundary over PR #145 (spec-only; diff origin/main...HEAD at 74ed09f: FR-150..FR-153, TC-160..TC-163, spec/tests.md). `quire validate --scope .` passes the eight new artifacts; spec/tests.md fails only on the `Coverage Status` vs `Status` header assertion, which origin/main fails too. Upstream checked at quire-specification origin/main (FR-060, FR-216, FR-217, FR-241, FR-242, FR-243, FR-290, FR-292, FR-351) and quire-spec-language origin/main (ADR-014, ADR-020, ADR-023 vocabulary).

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) declares `review_policy.mode: require` but is `status: proposed`, so it is not an active profile and no enforced ix-flow run is claimed. Direct analyses run: base, integrity, ears-conformance, scope-boundary, dependency, failure-domain, evidence.

## Examined

- FR-151 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)
- FR-151-AC-3 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)
- FR-153 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- FR-153-AC-4 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- FR-152 (spec/functional/FR-152-carry-projection-and-realizability-witness.md)

## Verdict

PASS WITH FINDINGS. The PR is consistent with merged FR-057, FR-058 and FR-301-AC-6. It uses no parallel vocabulary, keeps FR-302 to realizability only, and carries local-projection as a projected artifact with FR-030 correspondence. The unsupported dispositions map to truth and basis `unavailable` exactly as QSpec FR-241/242/243 derive. The findings below are defects only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | ADR-010 G2 puts the 'non-conclusive outcome on missing premises' inside QSL's realizability decision procedure, but FR-151 has the library refuse the disposition before dispatch with no backend call. Upstream admission already refuses a channel with a missing component (QSpec FR-054 through FR-216) and a role with absent internality (FR-217 Absence), so a checked package read through FR-044 cannot be in that state. AC-3's 'a request from which one channel premise or one role internality is removed' does not say whether the missing premise is in the requested premises or the read view. Name the seam where a premise can be absent, and record whether QSL's procedure or this library owns the missing-premise outcome. | spec/functional/FR-151-bind-communication-and-visibility-premises.md:49 |
| FND-002 | low | FR-153 calls `unsupported-requested-capability` 'QSL's cause name' and its warning names only the claim kind. QSpec FR-290's empty-candidate row and ADR-010 spell it `unsupported_projection`/`unsupported-requested-capability`, and that row's warning also names the item's requirements and the named backend when given. This matches merged FR-057's wording, so the gap is low, but AC-4 can pass with a warning that omits a named backend FR-290 requires. | spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md:51 |
