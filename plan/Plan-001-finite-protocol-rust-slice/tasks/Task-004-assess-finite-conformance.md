---
id: Task-004
title: "Assess finite global conformance"
type: Task
status: blocked
track: A
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-002, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-003, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-004, type: verifies }
---
# Task-004: Assess Finite Global Conformance

## Scope

Implement bounded batch and settled-incremental conformance over the finite graph,
compensation facts, independent decision-scope and surrounding-execution
progress/closure, completeness authority, and declared complete-global-conformance
closure.

## Subtasks

- [ ] Write TC-004 batch/incremental equivalence and legal-interleaving model cases first.
- [ ] Report unexpected, missing, misordered, duplicate-effect, cross-workflow, and compensation violations against exact obligations.
- [ ] Keep decision-scope progress/closure, surrounding-execution
  progress/closure, complete-global-conformance closure, and open, incomplete,
  unsupported, failed and exhausted outcomes independently observable.
- [ ] Ensure decisive open-subject results require the selected temporal rule and all declared decision support.
- [ ] Exercise every boundary with deterministic scheduling and no hidden wall-clock ordering premise.

## Deliverables

- Finite conformance evaluator, incremental state, obligation findings, TC-004, and trace bindings.

## Notes

- Unblocks Task-005.
