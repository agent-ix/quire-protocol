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

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | medium | The fix makes the deciding element identity resolve to one linked-graph node and refuses the disposition otherwise. QSpec FR-351 defines the deciding element as 'the exact value that decided the result' (a String), with no mapping to a protocol graph node, and ADR-010 states the TL-10 join as 'trace-position correspondence to the linked graph'. The node mapping is one QSpec does not derive, and it can refuse a valid QSL witness whose deciding element is a value. Join the component ADR-010 names (the trace position's step or event identity), or cite the owner text that makes the deciding element a node identity. | spec/functional/FR-152-carry-projection-and-realizability-witness.md:33 |


## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-005 | medium | FR-152 now resolves a deciding quantifier's `node_id`, and AC-1 and AC-4 test a `realizability` witness that has one. QSpec FR-351-AC-1 gives a witness a deciding quantifier only 'where the decision is a collection quantifier's', and FR-351's family-neutrality section lists none for the protocol family. FR-302, which FR-152 carries, imports only deciding element identity, index, value path and trace position, with no quantifier component. So AC-1's fixture is a witness shape that FR-302 does not carry for a realizability claim, and the node_id rule extends FR-302's carried record locally. Drop the quantifier branch: AC-1 then mirrors AC-2, with the witness carried byte-for-byte and the claim's source identities retained, and AC-4 keeps only its no-source-identity case. Alternatively, cite the owner text that gives a protocol witness a quantifier. | spec/functional/FR-152-carry-projection-and-realizability-witness.md:48 |
| FND-006 | low | The FR-152 statement ('every source identity it names joined to the FR-002 linked graph') and Outputs ('with each source identity it names resolved to one node of the linked graph') still describe the old rule, under which the witness names node identities. The new Behavior and the FR-351 note say the witness names none except an optional quantifier `node_id`, and the join runs through the source identities the disposition retains under FR-030. Bring the Description and Outputs in line with that. Outputs now reads: The disposition carrying the backend's witness or counterexample unchanged, with each source identity it names resolved to one node of the linked graph; for a `realizability` claim settled on `decisive-witness` or `decisive-counterexample`, the trace-position witness under FR-302; or one FR-030 refu | spec/functional/FR-152-carry-projection-and-realizability-witness.md:19 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Verified at PR head ca84709. |
| FND-002 | fixed | Verified at PR head ca84709. |
| FND-003 | fixed | Verified at PR head ca84709. |
| FND-004 | fixed | Verified at PR head df47610 (round 2). |
