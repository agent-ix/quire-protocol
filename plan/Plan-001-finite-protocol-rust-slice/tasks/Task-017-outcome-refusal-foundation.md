---
id: Task-017
title: "Establish the shared operation outcome and refusal foundation"
type: Task
status: done
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
`OperationOutcome<T>`, `ClaimOutcome<Row>`, `ResourceIncomplete`,
`ProtocolRefusal`, the seven closed domains, ten closed stages, their exact
stage/domain map, local/source cause separation and private construction. This
task owns no subsystem-specific
failure routing and no canonical result assembly; Task-006 closes those after
the semantic core exists.

## Subtasks

- [x] Add TC-011 red tests for the three operation outcomes, pairwise
  non-substitution, all valid stage/domain pairs and every invalid pair.
- [x] Implement `Completed(T)`, `Refused(ProtocolRefusal)` and
  `ResourceIncomplete(ResourceIncomplete)` without representing resource
  exhaustion as a refusal cause.
- [x] Implement `ClaimOutcome<Row>` so one claim retains either one canonical
  row or one claim-local refusal without suppressing sibling outcomes.
- [x] Implement the Protocol-local `quire.protocol/refusal/1` domains, stages
  and causes plus exact retained source causes and
  `unsupported-source-contract`.
- [x] Make all refusal, source-cause and resource constructors crate-private;
  validate their inputs before a value can cross the public boundary. No caller-built
  envelope, source cause or resource usage can cross the public boundary.
- [x] Move existing refusal catalog users onto the foundation without changing
  subsystem semantics. A crate-private adapter retains their typed local code
  as the non-empty legacy axis; Task-006 replaces those residual per-module
  routes with composed scenario axes and exact cause routing.
- [x] Record trace bindings for every new test; this type-only task adds no
  allocation, retention, traversal, operation, or output site and therefore no
  resource-inventory row.

## Deliverables

- One shared public outcome surface and private refusal/resource construction
  foundation, with TC-011 coverage and no competing non-success type in the
  composed Protocol API.

## Boundary

No canonical row, Engine, resolver, backend, E02 adapter or evidence store is
introduced here. Source-owner codes remain nested under their exact selected
contract and are never normalized into Protocol-local causes.

GitHub issue #62 owns this task.
