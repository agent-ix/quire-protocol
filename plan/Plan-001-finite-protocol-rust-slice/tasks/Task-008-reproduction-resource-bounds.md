---
id: Task-008
title: "Enforce reproduction and resource bounds"
type: Task
status: in_progress
track: Cross-Cutting-Closure
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-007, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-009, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-020, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-013, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-019, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/NFR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-008, type: verifies }
  - { target: ix://agent-ix/quire-protocol/TC-009, type: verifies }
---
# Task-008: Enforce Reproduction and Resource Bounds

## Scope

Apply semantic reproduction and every authored protocol and caller-supplied
resource guard across the complete admission, linking, evaluation, result, and
projection path.

## Subtasks

- [ ] Write TC-008 equivalent-presentation properties that preserve authored order and independently discriminate every named source/profile/model/protocol/request/binding/input/observation/progress/closure/producer/implementation/backend identity.
- [x] Build and bidirectionally reconcile the source-bound resource inventory, including each interface/site, authority, charge point, termination argument, typed incomplete code, and assigned boundary case.
- [ ] Write TC-009 below/at/above cases for every authored maximum and caller-supplied inventoried definition, graph, role, message, branching, iteration, active-instance, observation, lineage, result, derived-artifact, output, operation, allocation, and deadline limit.
- [x] Separate prechecked count/accounted-allocation refusal from monotonic deadline observation and report measured overshoot honestly.
- [ ] Kill every isolated inventory, guard-removal, guard-direction, and identity-canonicalization mutant for the intended reason.
- [x] Prove with action-capable doubles that assessment executes no business action or compensation.
- [x] Record deterministic seeds, toolchain identities, and corpus identities needed for local reproduction.
- [ ] Reconcile the completed test symbols against NFR-001-AC-1..AC-3 and NFR-002-AC-1..AC-4 with bare `ix-trace-rs` bindings.
- [ ] Internalize the unused parallel `CountGuard`/`RunGuard` public surface so
  `ResourceAccountant` is the one supported Protocol request accountant.
- [x] Retain external-owner usage under its exact accounting version. For QSL
  at `f1700a9`, consume `protocol_artifact::work::Dimension::accumulation()`
  directly and remove the local mirrored accumulation enum/table; never sum an
  owner rule locally.

## Deliverables

- Cross-cutting resource guard, reproduction suite, mutation ratchets, TC-008, TC-009, and trace bindings.

## Notes

- Whole-process containment remains an Engineering Assurance responsibility.
- Unblocks the final assurance gate.
