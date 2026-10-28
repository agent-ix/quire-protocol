---
id: SR-004
title: "Dependency review of the initial protocol implementation specification"
type: SpecReview
analysis: dependency
scope: "StR-001, FR-001..FR-008 and NFR-001..NFR-003 at 4e8f3e66a92069d90011b962c0819131dcd73a29"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-001, type: reviews }
---
## Summary

The local graph is acyclic. Admission and representation are enablement;
compensation, conformance, claim dispatch, canonical results, E02 projection,
and refusals follow in that order. External candidate, consumer, producer,
license, and evidence dependencies are explicit gates rather than local fallbacks.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Accepted external gate: implementation cannot begin until A mints an immutable candidate that pins the integrated FR-061/FR-062 bytes; existing `1-draft.3` predates them. | FR-001; IT-001; AD-001 |
| FND-002 | high | Accepted integration gates: F's consumer must preserve canonical result facts and Engineering Assurance issue 34 must publish typed Rust execution/transport before IT-002/IT-003 can run; neither absence authorizes a local implementation. | IT-002; IT-003; NFR-003 |
| FND-003 | medium | Accepted repository gate: the exact outbound SPDX choice and compatibility review precede implementation source or dependencies. | AD-001 |

## Classification and order

| Requirement | Class | Direct prerequisites |
| --- | --- | --- |
| FR-001 | Enablement | Accepted upstream candidate |
| FR-002 | Enablement | FR-001 |
| FR-003 | Feature | FR-002 |
| FR-004 | Feature | FR-002, FR-003 |
| FR-005 | Feature | FR-004 |
| FR-006 | Enablement | FR-004, FR-005 |
| FR-007 | Feature | FR-006, accepted E02 |
| FR-008 | Cross-cutting | FR-001, FR-007 |
| NFR-001 | Cross-cutting | FR-001, FR-004, FR-006, FR-007 |
| NFR-002 | Cross-cutting | FR-001, FR-002, FR-004 |
| NFR-003 | Cross-cutting | Whole repository and accepted assurance seams |
