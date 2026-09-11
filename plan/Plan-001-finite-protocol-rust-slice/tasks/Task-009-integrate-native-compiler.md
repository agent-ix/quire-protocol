---
id: Task-009
title: "Integrate the native compiler artifact"
type: Task
status: in_progress
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
- [ ] Mutate version, digest, feature, binding, baseline-selection state, numeric
  representation, and digest domain independently and prove refusal precedes evaluation.
- [ ] Inventory the dependency graph to prove no second frontend participates.

## Deliverables

- Pinned compiler contract, Rust IT-001 harness and fixtures, negative controls, and trace bindings.

## Notes

- A's accepted `quire.compiled-protocol/1` contract, public Rust reader and
  four-unit producer recipe are available at `quire-spec-language` commit
  `84975dd687d38d9b24eae84b7356e3f24cdc7b42` (merged PRs 52, 55 and 58).
- This increment consumes that boundary directly. It does not take ownership of
  later protocol conformance, execution, compensation assessment or E02 work.
- It supplies a fixture-driven smoke lane for positive admission plus selected
  one-axis seal, producer identity, feature and numeric refusals. The lane is
  deliberately tagged to IT-001/Task-009 only: its producer fixture retains
  A's draft wire labels, while the frozen-baseline FR acceptance cases remain
  open work under this task.
