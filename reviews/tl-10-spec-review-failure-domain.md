---
id: SR-2127030
title: "TL-10 FR-150..FR-153: failure-domain"
type: SpecReview
analysis: failure-domain
review_set: subset
scope: "agent-ix/quire-protocol; TL-10; PR #145 head 74ed09faaa3852ad3c4cd956fc24e2f3950e434b; spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md; spec/functional/FR-151-bind-communication-and-visibility-premises.md; spec/functional/FR-152-carry-projection-and-realizability-witness.md; spec/functional/FR-153-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/test-cases/TC-160-dispatch-projection-and-realizability-to-qsl.md; spec/test-cases/TC-161-bind-communication-and-visibility-premises.md; spec/test-cases/TC-162-carry-projection-and-realizability-witness.md; spec/test-cases/TC-163-settle-unsupported-while-no-qsl-backend-serves-the-kind.md; spec/tests.md"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-150, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-151, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-152, type: reviews }
---
## Summary

Ticket: TL-10. failure-domain over PR #145 (spec-only; diff origin/main...HEAD at 74ed09f: FR-150..FR-153, TC-160..TC-163, spec/tests.md). `quire validate --scope .` passes the eight new artifacts; spec/tests.md fails only on the `Coverage Status` vs `Status` header assertion, which origin/main fails too. Upstream checked at quire-specification origin/main (FR-060, FR-216, FR-217, FR-241, FR-242, FR-243, FR-290, FR-292, FR-351) and quire-spec-language origin/main (ADR-014, ADR-020, ADR-023 vocabulary).

## Assurance Context

AP-001 (spec/assurance/AP-001-protocol-implementation.md) declares `review_policy.mode: require` but is `status: proposed`, so it is not an active profile and no enforced ix-flow run is claimed. Direct analyses run: base, integrity, ears-conformance, scope-boundary, dependency, failure-domain, evidence.

## Examined

- FR-150-AC-6 (spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md)
- FR-151-AC-3 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)
- FR-151-AC-6 (spec/functional/FR-151-bind-communication-and-visibility-premises.md)
- FR-152-AC-1 (spec/functional/FR-152-carry-projection-and-realizability-witness.md)
- FR-152-AC-2 (spec/functional/FR-152-carry-projection-and-realizability-witness.md)
- FR-152-AC-4 (spec/functional/FR-152-carry-projection-and-realizability-witness.md)

## Verdict

PASS WITH FINDINGS. The PR is consistent with merged FR-057, FR-058 and FR-301-AC-6. It uses no parallel vocabulary, keeps FR-302 to realizability only, and carries local-projection as a projected artifact with FR-030 correspondence. The unsupported dispositions map to truth and basis `unavailable` exactly as QSpec FR-241/242/243 derive. The findings below are defects only.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-152-AC-1/AC-2 require 'every source identity it names equals the identity of one node of the linked graph', but the QSpec FR-351 record's components are a deciding element (a value), a value path, a trace position (a string) and an optional deciding-quantifier occurrence key, none of which is required to be a linked-graph source identity. A witness that names no source identity passes the AC vacuously, so a missing join cannot fail it. Name the component(s) that join to the FR-002 graph (for example the trace position's step), and require each to resolve. | spec/functional/FR-152-carry-projection-and-realizability-witness.md:42 |
| FND-002 | medium | FR-151-AC-6's second clause ('no premise value in the recorded backend calls is absent from the checked package') runs on a request with a premise absent, where FR-151-AC-3 requires the stand-in to record no call, so it holds over an empty record and cannot fail. Its first clause follows from AC-3's FR-030 refusal (execution `refused`, so truth `unavailable` under FR-241/FR-242). Move the no-invented-premise check onto a request with every premise present, or drop the AC. | spec/functional/FR-151-bind-communication-and-visibility-premises.md:52 |
| FND-003 | low | FR-150 covers backend failure (`failed`) but not the tool-absence path QSpec FR-290-AC-8 defines for a routed backend: a missing or wrong-version tool records `unsupported`/`tool-unavailable` before the run, and `failed` with the same cause after a passing probe. Say whether that cause is carried unchanged under FR-150-AC-5, or add it to AC-6. | spec/functional/FR-150-dispatch-projection-and-realizability-to-qsl.md:54 |
