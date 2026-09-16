---
id: Task-006
title: "Produce canonical results and typed refusals"
type: Task
status: in-progress
track: Core
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-004, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-005, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-006, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-006, type: verifies }
  - { target: ix://agent-ix/quire-protocol/TC-011, type: verifies }
---
# Task-006: Produce Canonical Results and Typed Refusals

## Scope

Implement the one per-assessment `assessment::Engine` orchestration and
`AssessmentBundle` commit boundary; connect the implemented strict canonical
result, immutable lineage and complete Contract-IR view to typed subsystem
outputs; and complete the refusal envelope shared by every refusing operation.

## Subtasks

- [x] Write TC-006 and TC-011 table, topology, diagnostic-catalog, paired-cause, simultaneous-disposition, mutation, and strict-reader cases first.
- [x] Construct one Engine only from an exact linked assessment, run every stage
  into a private candidate, and commit the bundle only after all whole-request
  invariants pass; exact replay and every failed candidate leave the observable
  snapshot, canonical serialization and accounting semantics unchanged.
- [x] Consume Task-017's shared three-member `OperationOutcome<T>` and ordered
  per-admitted-claim `ClaimOutcome`, then implement `AssessmentBundle`; keep FR-241 semantic
  dispositions, claim-local construction refusals and pre-row resource
  incompleteness non-interchangeable.
- [ ] Represent truth, adequacy, decision-scope progress/closure,
  surrounding-execution progress/closure, complete-global-conformance closure,
  settlement, support, claim dispositions, limitations, and provenance as
  orthogonal typed fields. The result shape and fail-closed path exist, but the
  pinned QSL/QObs revisions expose no authenticated oracle/dependence support
  view from which Protocol can establish settled truth.
- [ ] Validate predecessor existence, acyclicity, no self-reference, immutable
  supersession, exact dependency/source identities, and refusal of every
  progress/closure scope collapse or cross-wire. Result-owner lineage validates,
  but applicable correction remains blocked on an owner-authenticated
  contradiction/correction capability.
- [x] Independently omit, duplicate, cross-wire, scope-mutate, and
  identity-mutate every required execution, branch, and workflow global-closure
  premise while holding an independently settled per-obligation truth fixed.
- [x] Use the single RFC 8785 implementation only for owner-selected JCS
  domains; retain componentwise keys and exact producer bytes in their own domains.
- [x] Implement and mutation-test the exact reversible
  `quire.protocol.runtime-reference/1` carrier for opaque runtime identities in
  string-only result fields.
- [x] Make refusal identity independent of rendering while preserving the exact diagnostic catalog, stable domain/source code, originating stage/prerequisites, explicit location availability, paired causes, simultaneous dispositions, and cause order.
- [x] Refuse catalog drift or erased source stages and prove that repeated display names cannot invent a missing source location.

## Deliverables

- Per-assessment Engine, AssessmentBundle and composed use of the shared outcome algebra; canonical
  result/refusal types; strict readers; JCS and runtime-reference identity
  layers; lineage validator; TC-006/TC-011; and trace bindings.

## Notes

- Task 007 completed its total projection over the validated rows the public
  Engine can currently produce. Task 010 still owns the remaining
  observation/temporal integration and cannot credit authenticated support or
  correction scenarios until the owner capabilities above are delivered.
- Source catalogs remain owner-controlled. Protocol can complete its envelope
  without inventing a catalog by retaining a known source cause exactly and
  returning `unsupported-source-contract` for an unknown catalog/version.
