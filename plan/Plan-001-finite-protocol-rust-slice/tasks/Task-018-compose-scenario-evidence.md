---
id: Task-018
title: "Compose final scenario evidence from completed subsystem outcomes"
type: Task
status: planned
track: Qualification
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-006, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-008, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-013, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-017, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-005, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-016, type: verifies }
---
# Task-018: Compose Final Scenario Evidence from Completed Subsystem Outcomes

## Scope

Compose the Protocol-owned FR-017 value only after the final resolution,
operation/claim outcome and resource-accounting types exist. Validate the
integration-level refusal-axis distinctness invariant over complete record sets.
The value is passed onward but never stored. This task owns the selected Quoin
handoff adapter once `agent-ix/quoin#519` publishes it; Task-011 executes the
unchanged-head qualification run.

## Subtasks

- [ ] Write TC-016 over complete and deterministic partial resolution states,
  every operation/claim outcome, every accountant owner/version/dimension and
  compiler-checked trace binding.
- [ ] Consume the exact Task-013 capabilities/resolution state, Task-006 outcome
  identities and Task-008 resource records without declaring local mirrors.
- [ ] Refuse missing, duplicate, cross-wired or substituted owner/version,
  outcome, budget and trace components.
- [ ] Check refusal-axis injectivity over an integration record set; allow
  scenarios sharing one axis and reject two axes sharing one retained interior.
- [ ] Expose only a transient handoff value; add no storage, read-back, index,
  score, sufficiency or disposition operation.
- [ ] Record source-inventory entries and trace bindings for the composer.

## Deliverables

- Final scenario evidence composer, record-set distinctness validator, selected
  Quoin handoff adapter and TC-016 ready for Task-011 qualification.

## Boundary

This task does not execute producers or own Quoin storage/indexing. It does not close an
integration whose required contract remains unresolved, and it never converts
an unexecutable scenario into pass or fail.

GitHub issue #63 owns this task. Quoin issue #519 owns the external intake
contract.
