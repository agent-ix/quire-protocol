---
id: SR-013
title: "Scope review of the compiled protocol artifact gate"
type: SpecReview
analysis: scope-boundary
scope: "FR-001, IT-001, TC-001 and AD-001 at aba7c67506be8a82c2406a755fc5766dd5c09b13"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-001, type: reviews }
---
## Summary

A and the native compiler own the versioned compiled-protocol artifact contract.
`quire-protocol` owns strict consumption and semantic evaluation after acceptance.
The boundary prevents local parsing while preserving exact upstream authority.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Resolved: without an explicit producer contract, the consumer would have to parse native source, interpret prose definitions, or invent a private wire shape. All three violate the assigned boundary. | FR-001; IT-001; NFR-003 |
| FND-002 | low | The producer must supply artifact bytes, contract/producer/source revisions, selected-rule digests, dependency closure, profile, features, and source identities; the consumer recomputes and retains them without changing their meaning. | FR-001-AC-1..AC-3 |
| FND-003 | low | No ownership changes apply to temporal/observation production, Engineering Assurance execution, Quoin evidence, E02, or human disposition. | AD-001; IT-002; IT-003 |
