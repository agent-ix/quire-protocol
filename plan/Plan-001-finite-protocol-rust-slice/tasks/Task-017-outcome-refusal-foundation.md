---
id: Task-017
title: "Establish the shared operation outcome and refusal foundation"
type: Task
status: planned
track: Cross-cutting Foundation
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-016, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-014, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-016, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-011, type: verifies }
---
# Task-017: Establish the Shared Operation Outcome and Refusal Foundation

## Scope

Create the one cross-cutting type foundation every later public subsystem uses:
`OperationOutcome<T>`, `ResourceIncomplete`, `ProtocolRefusal`, the seven closed
domains, ten closed stages, their exact stage/domain map, local/source cause
separation and private construction. This task owns no subsystem-specific
failure routing and no canonical result assembly; Task-006 closes those after
the semantic core exists.

## Subtasks

- [ ] Add TC-011 red tests for the three operation outcomes, pairwise
  non-substitution, all valid stage/domain pairs and every invalid pair.
- [ ] Implement `Completed(T)`, `Refused(ProtocolRefusal)` and
  `ResourceIncomplete(ResourceIncomplete)` without representing resource
  exhaustion as a refusal cause.
- [ ] Implement the Protocol-local `quire.protocol/refusal/1` domains, stages
  and causes plus exact retained source causes and
  `unsupported-source-contract`.
- [ ] Make all constructors crate-private or validating; no caller-built
  envelope, source cause or resource usage can cross the public boundary.
- [ ] Move existing refusal catalog users onto the foundation without changing
  subsystem semantics; Task-006 later removes the residual per-module routes.
- [ ] Record resource inventory entries and trace bindings for every new site.

## Deliverables

- One shared public outcome surface and private refusal/resource construction
  foundation, with TC-011 coverage and no competing non-success type.

## Boundary

No canonical row, Engine, resolver, backend, E02 adapter or evidence store is
introduced here. Source-owner codes remain nested under their exact selected
contract and are never normalized into Protocol-local causes.

GitHub issue #62 owns this task.
