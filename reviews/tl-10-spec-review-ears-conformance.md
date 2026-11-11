---
id: SR-2127027
title: "TL-10 FR-150..FR-153: ears-conformance"
type: SpecReview
analysis: ears-conformance
review_set: subset
scope: "agent-ix/quire-protocol; TL-10; PR #145 head 74ed09faaa3852ad3c4cd956fc24e2f3950e434b; spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md; spec/functional/FR-151-bind-communication-and-visibility-premises.md; spec/functional/FR-152-carry-projection-and-realizability-witness.md; spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/test-cases/TC-160-dispatch-projection-and-realizability-to-qsl.md; spec/test-cases/TC-161-bind-communication-and-visibility-premises.md; spec/test-cases/TC-162-carry-projection-and-realizability-witness.md; spec/test-cases/TC-163-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-150, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-151, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-152, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-153, type: reviews }
---
## Summary

Ticket: TL-10. ears-conformance over PR #145 (spec-only; diff origin/main...HEAD at 74ed09f: FR-150..FR-153, TC-160..TC-163, spec/tests.md). `quire validate --scope .` passes the eight new artifacts; spec/tests.md fails only on the `Coverage Status` vs `Status` header assertion, which origin/main fails too. Upstream checked at quire-specification origin/main (FR-060, FR-216, FR-217, FR-241, FR-242, FR-243, FR-290, FR-292, FR-351) and quire-spec-language origin/main (ADR-014, ADR-020, ADR-023 vocabulary).

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) declares `review_policy.mode: require` but is `status: proposed`, so it is not an active profile and no enforced ix-flow run is claimed. Direct analyses run: base, integrity, ears-conformance, scope-boundary, dependency, failure-domain, evidence.

## Examined

- FR-150 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-151 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)
- FR-152 (spec/functional/FR-152-carry-projection-and-realizability-witness.md)
- FR-153 (spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md)
- FR-151-AC-5 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)

## Verdict

PASS WITH FINDINGS. The PR is consistent with merged FR-057, FR-058 and FR-301-AC-6. It uses no parallel vocabulary, keeps FR-302 to realizability only, and carries local-projection as a projected artifact with FR-030 correspondence. The unsupported dispositions map to truth and basis `unavailable` exactly as QSpec FR-241/242/243 derive. The findings below are defects only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-151 Behavior 'When QSL reports an outcome that is not conclusive' and FR-151-AC-5 'a non-conclusive result' use a term neither QSL nor QSpec defines. QSL's verdict is `Inconclusive` (V-6, with `InconclusiveCause`); QSpec's are truth `pending`/`unavailable` (FR-242) and basis `unsettled` (FR-243). The trigger set is open, so two implementers can pick different outcomes. Name QSL's `Inconclusive` verdict (and the QSpec values it maps to) as the trigger. | spec/functional/FR-151-bind-communication-and-visibility-premises.md:51 |
| FND-002 | low | The FR-152 statement fires 'When QSL's backend returns a witness or counterexample for a `local-projection` or `realizability` claim', but a local-projection result is a projected local artifact (FR-152 Inputs, AC-5), not a witness, and FR-302 covers only realizability and refinement. Split the trigger: a projected artifact for local-projection, a witness or counterexample for realizability. | spec/functional/FR-152-carry-projection-and-realizability-witness.md:19 |
