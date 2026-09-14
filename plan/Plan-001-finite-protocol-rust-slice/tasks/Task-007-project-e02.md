---
id: Task-007
title: "Project canonical results to E02"
type: Task
status: blocked
track: Producer-Integration
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-006, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-007, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-007, type: verifies }
---
# Task-007: Project Canonical Results to E02

## Scope

Implement the selected FR-062 total projection into the exact E02 strict-reader
contract, preserving the canonical source and every mapped finding.

## Subtasks

- [ ] Write TC-007 for every source row, simultaneous finding set, and refusal condition first.
- [ ] Pin source, target, adapter, mapping, and dependency versions and compute the derived identity with RFC 8785.
- [ ] Preserve the exact canonical result reference, oracle lineage, native artifacts, and limitations.
- [ ] Keep protocol adequacy distinct from E02 adequacy and any later sufficiency disposition.
- [ ] Invoke the E02 strict reader and refuse finding loss, unknown versions, or any non-total mapping without emitting a `TechniqueResult`.

## Deliverables

- E02 projection adapter, version pins, strict-reader integration, TC-007, and trace bindings.

## Notes

- Unblocks Task-008 and the E02 half of Task-011.
- qverify implements the exact draft-2 reader, but its owner documentation still
  calls the cross-repository seam provisional. Do not implement against a
  substitute; proceed only after `agent-ix/quire-verification#32` reconciles and
  accepts that revision.
