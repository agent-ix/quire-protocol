---
id: Task-005
title: "Dispatch independent protocol claims"
type: Task
status: done
track: Core
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

The closed disposition and premise kernels already exist. This task makes the
conformance report their required production input and removes the arbitrary
caller backend callback from the supported path.

## Subtasks

- [x] Write TC-005 controls for every supported and unsupported claim combination first.
- [x] Define separate conformance, monitorability, projection, realizability, refinement, and composition dispositions.
- [x] Reject circular assumptions and backend identity/version mismatches before dispatch.
- [x] Retain and strictly match each claim's exact admitted fragment, backend,
  assumptions, communication/visibility/environment premises, source identities,
  and derived-artifact correspondence.
- [x] Preserve finite conformance when another requested claim is unsupported or fails.
- [x] Seal backend dispatch to accepted local implementations and verify no
  local heuristic, caller callback or alternate backend converts a missing
  capability into success.

## Deliverables

- Claim request/disposition types, finite dispatch, unsupported boundary, TC-005, and trace bindings.

## Notes

- Advanced algorithms remain outside this plan until E03 admission.
- Unblocks Task-006.
- SR-114/SR-115 record the passing scoped Rust review and gap analysis.
