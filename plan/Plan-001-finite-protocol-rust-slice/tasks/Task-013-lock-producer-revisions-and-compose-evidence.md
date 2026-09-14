---
id: Task-013
title: "Resolve external producer revisions into private capabilities"
type: Task
status: in_progress
track: Core
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-016, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-017, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-009, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-013, type: verifies }
---
# Task-013: Resolve External Producer Revisions into Private Capabilities

## Scope

Own the Protocol-owned FR-009 enablement: manifest resolution, request-bound
private resolved capabilities and TC-013. Concrete entry population belongs to
Tasks 009 through 011; final scenario-record composition belongs to Task-018
after the outcome/resource model is complete. This task therefore closes in one
PR without depending on the integrations it enables.

This task exists because those scenarios and requirements were authored with no
owning task, which is the condition SR-040 recorded for the `/2` lane and SR-047
recorded here.

## Split

FR-009 divides cleanly, and the split is the point: the resolution mechanism is
Protocol-owned and unblocked, while only moving an entry from `unresolved` to `pinned`
waits on anyone else. Scheduling the whole requirement behind seven external
acceptances would leave the integrations with nothing to resolve against.

### Protocol-owned, unblocked — no external delivery required

- [ ] Author the version-lock manifest artifact. None exists in the tree today,
  so every scenario that presumes resolution currently presumes a mechanism that
  is unbuilt.
- [ ] Implement resolution: the two-status model (`pinned`, `unresolved` with a
  reason), the `(owner, contract version)` key, per-key typed refusal, duplicate
  key refusal before resolution, and entry independence.
- [ ] Implement digest verification over the exact delivered bytes with no
  canonicalization, passing the verified byte sequence through to the consumer,
  and refusing an interface-only entry that carries a digest.
- [ ] Implement the resolution budget over entry count, bytes digested and a
  deadline, returning resource-incomplete rather than a refusal.
- [ ] Prove containment: resolution reads no version-control history, spawns no
  process, opens no network connection.
- [ ] Implement TC-013 against Protocol-owned fixtures.
- [ ] Mint constructor-private `ResolvedArtifact<C>` and
  `ResolvedInterface<C>` only after all exact checks; bind each capability to
  the request/resolution identity, selected key/revision and verified bytes or
  adapter-compiled interface selection so it cannot authorize another request.
- [ ] Validate required keys in deterministic lexicographic order and prove
  offered-manifest permutation changes neither the resolved set nor the first
  cause/partial state.

### Consuming-task responsibilities

- Task-009 records its concrete QSL entries, Task-010 its QObs/QSL temporal
  entries, and Task-011 its Engineering Assurance/E02/Quoin entries. Task-018
  composes their final evidence values.
- A consuming task whose owner contract is unresolved records the exact blocker
  and no capability; that does not block this mechanism from closing.

## Deliverables

- The version-lock manifest artifact, request-bound capability resolution and
  TC-013.

## Boundary

Resolution indexes what was consumed; it never produces, canonicalizes, stores,
or assesses a producer artifact. No local evidence store, catalogue, sufficiency
decision, or human-disposition path. AD-001's qualification-evidence intake
section is normative.

Completing this task closes only the Protocol-owned resolver. FR-009 integration
acceptance remains distributed to Tasks 009 through 011, and NFR-005/FR-017
evidence composition closes under Task-018. This task establishes no temporal
result, observation adequacy, replay agreement, evidence handoff or campaign
completion.
