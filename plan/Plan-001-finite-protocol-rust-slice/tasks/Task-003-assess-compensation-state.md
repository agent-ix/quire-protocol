---
id: Task-003
title: "Assess compensation state"
type: Task
status: blocked
track: A
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-002, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-003, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-003, type: verifies }
---
# Task-003: Assess Compensation State

## Scope

Implement pure activation, effect, registration, retry, commit, and recovery
assessment over the linked protocol and accepted producer facts.

## Subtasks

- [ ] Write TC-003 state-transition and adverse-path cases first.
- [ ] Preserve delivery, attempt, effect, registration, commit, and recovery identities as distinct types.
- [ ] Register compensation only after the paired successful effect and enforce the authored finite retry and commit ordering.
- [ ] Require exact population and completeness authority for aggregate partial recovery.
- [ ] Prove the module performs no business or compensation I/O and derives no fact from successful transport alone.

## Deliverables

- Pure compensation evaluator, typed outcome facts, TC-003, mutation controls, and trace bindings.

## Notes

- Unblocks Task-004.
