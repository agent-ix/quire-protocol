---
id: Task-009
title: "Integrate the native compiler artifact"
type: Task
status: done
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

- [x] Freeze the accepted QSL `6c8c401` compiler contract and owner-published
  `/1` and `/2` handoff revisions.
- [x] Implement IT-001 SC-01 through SC-05 as default-running Rust integration tests.
- [x] Retain `/1` as the declared capability-free compatibility exception and
  leave final scenario-value composition to Task-018; normal full-assessment
  integrations still require Task-013 capabilities.
- [x] Retain every producer identity and source reference through the public API.
- [x] Round-trip field-boundary and admitted signed-64/exact-rational values through
  the accepted versioned numeric wire profile without loss or alternate form.
- [x] Mutate version, digest, feature, binding, baseline-selection state, numeric
  representation, and digest domain independently and prove refusal precedes evaluation.
- [x] Bind each IT-001 public-API case to the FR-001 acceptance criterion it
  evidences, run `quire coverage`, and record the installed engine's declared
  integration-ID limitation (`quire-rs#244`) rather than misreporting an
  integration-only criterion as engine-bound.
- [x] Inventory the dependency graph and first-party source path positively to prove
  that the A-owned public reader is the only native frontend; a denylist of
  foreign-runtime crates is not sufficient.
- [x] Execute the local version-1 linker and the SC-06a before/after
  compatibility comparison; admission alone is not link evidence.

## Deliverables

- Pinned compiler contract, Rust IT-001 harness and fixtures, negative controls, and trace bindings.

## Notes

- The accepted `/1` and `/2` readers and committed handoffs are consumed at QSL
  `6c8c401`. QSL #112 published the `/1` address and #114 corrected its distinct
  role authorities; both are closed.
- This increment consumes that boundary directly. It does not take ownership of
  later protocol conformance, execution, compensation assessment or E02 work.
- The default lane passes all 24 current tests with no ignores and performs
  `intake::v1::admit` plus `model::v1::link_static`. SC-06a separately passes
  the identical 23-case historical corpus at Protocol `0e5ec292` and `5ce9070e`.
