---
id: Task-009
title: "Integrate the native compiler artifact"
type: Task
status: blocked
track: B
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-002, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/IT-001, type: verifies }
---
# Task-009: Integrate the Native Compiler Artifact

## Scope

Exercise the accepted A-owned compiler artifact through admission and linking
without adding a parser, source frontend, numeric reinterpretation, digest-domain
substitution, or identity reinterpretation.

## Subtasks

- [ ] Freeze the exact accepted compiler artifact contract and fixture revisions.
- [ ] Implement IT-001 SC-01 through SC-05 as Rust integration tests.
- [ ] Retain every producer identity and source reference through the public API.
- [ ] Round-trip field-boundary and admitted signed-64/exact-rational values through
  the accepted versioned numeric wire profile without loss or alternate form.
- [ ] Mutate version, digest, feature, binding, candidate state, numeric
  representation, and digest domain independently and prove refusal precedes evaluation.
- [ ] Inventory the dependency graph to prove no second frontend participates.

## Deliverables

- Pinned compiler contract, Rust IT-001 harness and fixtures, negative controls, and trace bindings.

## Notes

- Blocked by Task-002 and the accepted successor compiler artifact contract.
- May proceed independently of Tasks 003 through 008 once those gates close.
