---
id: Task-015
title: "Complete the structural Contract-IR result-map view"
type: Task
status: in_progress
track: A-Enablement
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-042, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-015, type: verifies }
---
# Task-015: Complete the Structural Contract-IR Result-Map View

## Scope

Expose every already-validated progress/closure owner-reference field through
the constructor-private Contract-IR mapping view so downstream temporal joins
can compare exact owner structure without parsing mapping JSON or reconstructing
owner semantics.

## Subtasks

- [ ] Extend TC-015 first to require borrowed access to the selected owner
  contract, schema digest, assertion identity/digest, authority
  identity/revision/digest, scope identity, population identity, and state.
- [ ] Delegate the four missing accessors directly to the validated source
  `AxisView`, adding no parser, evaluator, trust flag, allocation, or fallback.
- [ ] Run the exact repository-local Rust gate and reconcile FR-042/TC-015
  traceability on the unchanged implementation head.

## Deliverables

- Complete `MappedAxisView` borrowed accessor surface, discriminating TC-015
  assertions, passing local gate, and PR-time Rust/code/gap review artifacts.

## Notes

- This task is independent owner-interface enablement for
  `quire-contract-ir#71` / FR-026, tracked by
  [quire-protocol#54](https://github.com/agent-ix/quire-protocol/issues/54)
  under protocol epic #14. It does not implement temporal semantics in Quire
  Protocol and does not advance the blocked IT-002 integration task.
- The validated source already retains and validates every field. This task
  exposes four omitted fields and must not duplicate validation or deserialize
  canonical mapping bytes.
