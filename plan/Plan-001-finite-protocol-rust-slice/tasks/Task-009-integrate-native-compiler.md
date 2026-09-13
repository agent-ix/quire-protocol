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
- [ ] Bind each IT-001 public-API case to the FR-001/FR-002 acceptance criterion it
  evidences, and prove the matrix credit with `quire coverage` rather than tags alone.
- [ ] Inventory the dependency graph and first-party source path positively to prove
  that the A-owned public reader is the only native frontend; a denylist of
  foreign-runtime crates is not sufficient.

## Deliverables

- Pinned compiler contract, Rust IT-001 harness and fixtures, negative controls, and trace bindings.

## Notes

- The accepted `quire.compiled-protocol/1` contract and public Rust reader are
  compiler-owned. PR72 candidate revision
  `5690c2b29c155ec72e21f5a7bb00fe1146b6f71a` regenerates the four-unit fixture
  with reachable signed-64/exact-rational endpoints at
  `/tmp/quire-native-handoff-signed64-endpoints/`; B must derive its retained
  selection from those producer inputs without pinning the rebuild-volatile ELF
  seal or reading `expected.json` as authorization.
- This increment consumes that boundary directly. It does not take ownership of
  later protocol conformance, execution, compensation assessment or E02 work.
- Review SR-024 through SR-032 requires criterion-level matrix bindings and a
  positive parser/reader inventory before this candidate lane can become
  qualification evidence. PR72 and `quire-spec-language#40` remain external
  acceptance/traceability blockers; their candidate fixture does not mint a
  promoted baseline.
