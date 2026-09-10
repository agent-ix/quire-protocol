---
id: Task-006
title: "Produce canonical results and typed refusals"
type: Task
status: blocked
track: A
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

Implement the strict canonical result, immutable lineage, RFC 8785 identity, and
complete typed refusal envelope shared by every public operation.

## Subtasks

- [ ] Write TC-006 and TC-011 table, topology, mutation, and strict-reader cases first.
- [ ] Represent truth, adequacy, decision-scope progress/closure,
  surrounding-execution progress/closure, settlement, support, claim dispositions,
  limitations, and provenance as orthogonal typed fields.
- [ ] Validate predecessor existence, acyclicity, no self-reference, immutable
  supersession, exact dependency/source identities, and refusal of every
  progress/closure scope collapse or cross-wire.
- [ ] Canonicalize every digest and identity comparison through one RFC 8785 implementation and reject unknown contract versions.
- [ ] Make refusal identity independent of rendering while preserving stable domain, source code, identity/location, and cause order.

## Deliverables

- Canonical result/refusal types, strict readers, JCS identity layer, lineage validator, TC-006, TC-011, and trace bindings.

## Notes

- Unblocks Tasks 007 and 010.
