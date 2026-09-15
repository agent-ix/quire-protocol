---
id: SR-094
title: "Risk and complexity review of the complete Protocol architecture"
type: SpecReview
analysis: risk-complexity
scope: "Complete remaining Protocol implementation campaign"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/AD-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: reviews }
---
# SR-094: Risk and complexity review of the complete Protocol architecture

## Summary

Technical volatility, semantic risk and integration complexity were evaluated
across the entire remaining campaign before implementation resumes.

## Verdict

**PASS after decomposition.** High-volatility boundaries are isolated behind
small typed adapters; the stable serial core does not wait on E02 or Quoin.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: cross-cutting outcome, capability and evidence types were scheduled inside later feature tasks. Tasks 017, 013 and 018 now isolate them at their actual dependency points. | Plan-001 | wrong-requirement |
| FND-002 | high | Resolved: batch/incremental and recovery extension seams permitted semantic forks. One conformance kernel and the admitted QSL evaluator path are now mandatory. | AD-001; Tasks 003/004 | wrong-requirement |
| FND-003 | medium | External volatility remains at qverify, Quoin and FR-236, but each is isolated to its adapter/mapping and cannot force redesign of the serial core. | AD-001 Risks | correct-requirement-no-evidence |

## Risk register

| Risk | Level | Control and owner |
| --- | --- | --- |
| Resolver capability can be forged/reused | high | private constructors plus request/value identity; Task-013 / TC-013 |
| QSL graph projection loses a future enum/member | high | exhaustive typed matching, no wildcard/JSON erasure; Task-002 / TC-002 |
| Batch and incremental semantics drift | high | one `ConformanceKernel`; Task-004 / TC-004 |
| Recovery semantics escape through a callback | high | open Task-003 work: replace the public callback with the admitted QSL expression/result capability; TC-003 must prove the production surface is closed |
| Fatal and claim-local failures collapse siblings | high | two-level outcome algebra; Tasks 017/006 / TC-011/006 |
| Two resource architectures disagree | high | one accountant and owner accumulation accessor; Task-008 / TC-009 |
| E02/Quoin owner contracts change | medium | isolated adapters remain unresolved; Tasks 007/018/011 do not invent substitutes |
| FR-236 outcome-domain mapping | medium | QSpec #57 accepted the partition at `478afa2`; retain attempt, effect, and recovery subjects separately while completing Task-003's typed-expression bridge |

The plan places the highest cross-cutting types in Tasks 017 and 013 before the
serial implementation chain, preventing later subsystems from inventing local
outcome, authority or budget models.
