---
id: SR-105
title: "Base review of the Protocol activation-binding integration"
type: SpecReview
analysis: base
scope: "QSpec FR-300, Protocol FR-004/FR-019, AD-001, IT-002, Tasks 003/004"
review_set: base
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-004, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-019, type: reviews }
  - { target: ix://agent-ix/quire-protocol/AD-001, type: reviews }
---
# SR-105: Base review of the Protocol activation-binding integration

## Summary

The complete assessment architecture now has one owner-defined bridge from a
linked control and admitted trigger to an event-triggered obligation. The
Protocol consumer contract is coherent; its producer interface is an explicit
upstream implementation dependency rather than permission to retain the legacy
caller-built model.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Open implementation dependency: the selected QSL revision does not yet publish a versioned typed FR-300 activation-binding interface. Task-003/004 cannot replace the legacy caller-built activation/model path until QSL publishes it; Protocol must refuse its absence and must not reconstruct a substitute. | QSpec FR-300; FR-019 Inputs/Behavior; FR-004 Inputs/Behavior; AD-001 Stateful assessment view; IT-002 | correct-requirement-no-evidence |

## Decision

The requirement and boundary are accepted. The next implementation work is the
QSL producer interface, followed by Protocol consumption and removal of the
legacy public construction path. No local shadow activation binding is allowed.
