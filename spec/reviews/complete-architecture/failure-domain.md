---
id: SR-090
title: "Failure-domain review of the complete Protocol architecture"
type: SpecReview
analysis: failure-domain
scope: "AD-001 failure topology; FR-003..FR-009; FR-014; FR-016; FR-024; FR-036..FR-041; NFR-001..NFR-003 and NFR-008"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/AD-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: reviews }
---
# SR-090: Failure-domain review of the complete Protocol architecture

## Summary

The complete pipeline was checked for extension-point failures, identity
confusion, impure evaluation and topology/order defects.

## Verdict

**PASS after remediation.** Extension failures, identity conflicts, evaluation
purity failures and graph/topology failures have explicit non-success outcomes.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: resource exhaustion was previously allowed to enter the refusal catalogue. `ResourceIncomplete` is now a sibling operation outcome and is never a `ProtocolCause`; claim-local resource incompleteness remains the exact FR-241 disposition. | FR-008; FR-014; FR-016; FR-039 | wrong-requirement |
| FND-002 | high | Resolved: recovery accepted a caller callback, permitting unreviewed semantics, I/O and nondeterminism. Production recovery now evaluates only the admitted typed QSL recovery expression through its resolved capability; test doubles remain crate-private. | FR-024; AD-001 Stateful assessment; Task-003 | wrong-requirement |
| FND-003 | high | Resolved: “transactional” could be misread as byte-equality of Rust allocation state. The observable invariant is now the prior typed snapshot, canonical serialization and accounting semantics plus successful replay of the same continuation. | AD-001 Stateful assessment; Tasks 002..006 | wrong-requirement |
| FND-004 | medium | Resolved: refusal ownership mixed source and local catalogues. The envelope has seven local domains and ten exact local stages; source contract, code, cause and locus remain a separate selected interior. | FR-008; FR-036; FR-038; ADR-008 | wrong-requirement |
| FND-005 | medium | Owner-blocked: QSpec FR-236 does not partition its five labels between per-attempt and aggregate facts. Only those Task-003 mapping cases remain unexecutable; Protocol must not infer the partition. | FR-003; FR-023; Task-003 | missing-requirement |

No blocked owner decision prevents Task-017, Task-013 or Task-002 from starting.
