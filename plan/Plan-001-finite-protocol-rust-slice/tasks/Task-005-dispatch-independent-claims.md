---
id: Task-005
title: "Dispatch independent protocol claims"
type: Task
status: blocked
track: A
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-004, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-005, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-005, type: verifies }
---
# Task-005: Dispatch Independent Protocol Claims

## Scope

Implement total per-claim dispatch for the selected finite slice while preserving
unsupported, failed, and independently established outcomes without fallback promotion.

## Subtasks

- [ ] Write TC-005 controls for every supported and unsupported claim combination first.
- [ ] Define separate conformance, monitorability, projection, realizability, refinement, and composition dispositions.
- [ ] Reject circular assumptions and backend identity/version mismatches before dispatch.
- [ ] Preserve finite conformance when another requested claim is unsupported or fails.
- [ ] Verify no local heuristic or alternate backend converts a missing capability into success.

## Deliverables

- Claim request/disposition types, finite dispatch, unsupported boundary, TC-005, and trace bindings.

## Notes

- Advanced algorithms remain outside this plan until E03 admission.
- Unblocks Task-006.
