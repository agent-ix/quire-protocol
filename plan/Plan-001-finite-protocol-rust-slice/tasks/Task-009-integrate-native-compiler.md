---
id: Task-009
title: "Integrate the native compiler artifact"
type: Task
status: in_progress
track: Producer-Integration
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-002, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-019, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-009, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-005, type: references }
  - { target: ix://agent-ix/quire-protocol/IT-001, type: verifies }
---
# Task-009: Integrate the Native Compiler Artifact

## Scope

Exercise the accepted A-owned compiler artifact through admission and linking
without adding a parser, source frontend, numeric reinterpretation, digest-domain
substitution, or identity reinterpretation.

## Subtasks

- [x] Freeze the accepted QSL `f1700a9` compiler contract and owner-published
  `/2` handoff revision.
- [ ] Implement IT-001 SC-01 through SC-05 as Rust integration tests.
- [ ] Resolve the exact `/1` or `/2` capability before each normal integration
  path and compose its Task-018 evidence record; exercise only SC-06a through
  the frozen non-integration `/1` facade and record its exact compatibility
  selection instead of pretending it has a capability.
- [ ] Retain every producer identity and source reference through the public API.
- [ ] Round-trip field-boundary and admitted signed-64/exact-rational values through
  the accepted versioned numeric wire profile without loss or alternate form.
- [ ] Mutate version, digest, feature, binding, baseline-selection state, numeric
  representation, and digest domain independently and prove refusal precedes evaluation.
- [ ] Bind each IT-001 public-API case to the FR-001/FR-002 acceptance criterion it
  evidences, and prove the matrix credit with `quire coverage` rather than tags alone.
- [ ] Inventory the dependency graph and first-party source path positively to prove
  that the A-owned public reader is the only native frontend; a denylist of
  foreign-runtime crates is not sufficient.
- [ ] Execute the local version-1 linker and the SC-06a before/after
  compatibility comparison; admission alone is not link evidence.

## Deliverables

- Pinned compiler contract, Rust IT-001 harness and fixtures, negative controls, and trace bindings.

## Notes

- The accepted `/1` and `/2` readers and published `/2` handoff are delivered at
  QSL `f1700a9`; QSL #40 is closed. The remaining blocker is the local exact
  linker and default-running `/1` corpus, not producer promotion.
- This increment consumes that boundary directly. It does not take ownership of
  later protocol conformance, execution, compensation assessment or E02 work.
- Review SR-024 through SR-032 still requires criterion-level matrix bindings
  and a positive parser/reader inventory before qualification evidence closes.
