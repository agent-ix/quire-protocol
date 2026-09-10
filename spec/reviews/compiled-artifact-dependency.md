---
id: SR-011
title: "Dependency review of the compiled protocol artifact gate"
type: SpecReview
analysis: dependency
scope: "FR-001, IT-001 and AD-001 at aba7c67506be8a82c2406a755fc5766dd5c09b13"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-001, type: reviews }
---
## Summary

The dependency graph now distinguishes semantic candidate adoption from the
compiler artifact contract required to construct typed Rust input. Both are
external enablement gates for admission; neither depends on local implementation.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Resolved: candidate adoption alone could appear to unblock Task-001 even though the candidate is prose and this repository prohibits a source parser. The accepted compiler contract is now a separate prerequisite. | FR-001; IT-001; AD-001 |
| FND-002 | low | The local order remains acyclic: accepted candidate plus artifact contract enable FR-001; FR-001 enables FR-002; IT-001 verifies the same producer boundary after linking exists. | FR-001; FR-002; IT-001 |
| FND-003 | low | The external gate does not make Engineering Assurance #34 a semantic dependency; that interface remains required only for producer execution and later integration evidence. | AD-001; IT-003 |
