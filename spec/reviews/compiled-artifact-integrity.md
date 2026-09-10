---
id: SR-012
title: "Integrity review of the compiled protocol artifact gate"
type: SpecReview
analysis: integrity
scope: "FR-001, IT-001, TC-001 and AD-001 at aba7c67506be8a82c2406a755fc5766dd5c09b13"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-001, type: reviews }
---
## Summary

The revised artifacts consistently require one accepted artifact contract,
producer revision, promoted candidate, dependency closure, and feature set. The
contract identity is checked before semantic evaluation and does not replace
the separately pinned formal-rule bytes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Resolved: FR-001 inputs, behavior, AC-1/AC-2, IT-001 preconditions/dependencies, TC-001 procedure, and AD-001 risk now describe the same two-gate boundary. | FR-001; IT-001; TC-001; AD-001 |
| FND-002 | low | No local artifact schema, parser, default version, nearest-version fallback, or source-to-graph reinterpretation was introduced. | FR-001-AC-3; NFR-003 |
| FND-003 | low | Matrix row identities and counts remain correct because the existing TC-001 and IT-001 cases own the strengthened dimensions. | TM-001 |
