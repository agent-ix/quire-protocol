---
id: SR-092
title: "Dependency review of the complete Protocol architecture"
type: SpecReview
analysis: dependency
scope: "AD-001 external contracts, FR-009, IT-001..IT-003, DEPENDENCIES.md and Plan-001 DAG"
review_set: all
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-009, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: reviews }
---
# SR-092: Dependency review of the complete Protocol architecture

## Summary

The plan DAG and every external contract were checked against current owner
tickets, merged code and exact repository pins.

## Verdict

**PASS after remediation.** Enablement precedes feature work and every external
contract is pinned, explicitly unresolved, or excluded from Protocol inputs.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: Task-013 combined early resolver enablement with late evidence composition. Task-018 now owns composition after the serial result and resource paths. | Task-013; Task-018; Plan-001 | wrong-requirement |
| FND-002 | high | Resolved: QCI #71 was listed as a Protocol prerequisite although it consumes Protocol output. The edge is now downstream only. | AD-001; IT-002; QCI #71/PR #78 | wrong-requirement |
| FND-003 | medium | Owner-blocked: qverify still calls E02 cross-repository use provisional and Quoin exposes no Protocol scenario intake. Their exact scenarios remain unresolved without blocking the serial core. | FR-009; Tasks 007/018/011 | missing-requirement |
| FND-004 | medium | Resolved upstream: QSpec #57 accepted FR-236 at `478afa2`, so Task-003 may implement the exact attempt/effect/recovery mapping without guessing a partition. | Task-003; FR-236; QSpec #57 | missing-requirement |

## Verified owner state

| Owner contract | Current fact | Plan consequence |
| --- | --- | --- |
| QSL compiled `/1`, `/2`, native temporal | exact Protocol pin `f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07` | available |
| QObs runtime/authority | accepted owner `93bc565`; Protocol pins and consumes that revision directly | available to Tasks 002/010 |
| Engineering Assurance request/result v1 | accepted by PR #46, merge `d9d3023` | available to Task-011 |
| QCI native/TL join | #71 / PR #78, merge `69ec82b` | downstream; no Protocol dependency |
| qverify E02 draft-2 | implementation exists; owner document still says provisional across repositories | Task-007 scenario remains unresolved |
| Quoin scenario intake | no operation accepts the Protocol record | Task-018/011 handoff remains unresolved |
| QSpec FR-236 partition | accepted by QSpec #57 at `478afa2` | implement exact Task-003 mapping; no owner block |

## Topological order

`016 -> 017 -> 013 -> 002 -> 003 -> 004 -> 005 -> 006`, followed by the
version/E02/observation integration branches, Task-008's final accountant sweep,
Task-018 evidence composition, and Task-011 qualification. Task-013 and
Task-018 are deliberately separate. No runtime manifest entry depends on
another entry; lexicographic validation is deterministic reporting, not causal
precedence.
