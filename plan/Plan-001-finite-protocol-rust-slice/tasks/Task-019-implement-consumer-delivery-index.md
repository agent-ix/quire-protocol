---
id: Task-019
title: "Implement the consumer-owned delivery index"
type: Task
status: planned
track: Producer Integration
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-013, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-018, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-006, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-017, type: verifies }
---
# Task-019: Implement the Consumer-Owned Delivery Index

## Scope

Implement the already-specified FR-018 consumer record for one pinned producer
delivery. This task owns the delivery-index behavior that TM-001 previously
credited to Task-013 even though neither that resolver nor code on `main`
implemented it. The correction creates implementation ownership for accepted
architecture; it does not add or redesign architecture.

## Subtasks

- [ ] Implement exact (`delivery_root`, `producer_revision`) identity with
  offer, input, temporal, limit and mutation state excluded from equality.
- [ ] Retain producer revision, artifact references and every indexed byte
  sequence without canonicalization, normalization or a synthesized digest.
- [ ] Enforce the accepted producer binding cardinality, no-repeat and ascending
  `declaration` rule without inventing a stronger upstream identity or order.
- [ ] Return distinct typed outcomes for missing root/index/member, unreadable
  index/member, ambiguous mutation, repeated declaration, unsorted bindings and
  unindexed input role; none is a pass or an empty default.
- [ ] Key limits by exact accountant owner/contract version and exact typed
  owner dimension, refuse duplicate keys, and retain deterministic non-causal
  ordering without Protocol/compiled-protocol substitution.
- [ ] Prove with TC-017 that editing or deleting the index
  supplies, selects, repairs or widens no version-lock entry and changes no
  resolver outcome.
- [ ] Add the final delivery-index source-inventory and resource sites before
  Task-008 performs the global reconciliation.

## Deliverables

- Protocol-owned `DeliveryIndex` values, validation/outcome types and TC-017.

## Boundary

The index records what the consumer observed in a selected delivery root. It is
not a producer manifest, transport observation, version-lock authority, digest
authority, store, discovery service or workflow runner. It performs no VCS,
network or process access and never repairs an unresolved contract.

GitHub issue #66 owns this task. It depends on Task-013/#49, may proceed in
parallel with Task-002, and must land before Task-009 consumes complete delivery
metadata and before Task-008's final inventory/reproduction sweep.
