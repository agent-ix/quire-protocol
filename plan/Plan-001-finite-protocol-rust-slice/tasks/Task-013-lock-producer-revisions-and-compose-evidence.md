---
id: Task-013
title: "Lock external producer revisions and compose per-scenario evidence"
type: Task
status: in_progress
track: B
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-009, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-005, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-013, type: verifies }
  - { target: ix://agent-ix/quire-protocol/IT-002, type: verifies }
  - { target: ix://agent-ix/quire-protocol/IT-003, type: verifies }
---
# Task-013: Lock External Producer Revisions and Compose Per-Scenario Evidence

## Scope

Own FR-009, NFR-005, TC-013, and the scenarios added to both integrations for
them: IT-002-SC-05a..SC-05d, SC-06, SC-07a, SC-07b and SC-08, and IT-003-SC-06,
SC-07, SC-08a, SC-08b and SC-09. Task-010 keeps IT-002-SC-01..SC-04 and Task-011
keeps IT-003-SC-01..SC-05; neither is changed by this task.

This task exists because those scenarios and requirements were authored with no
owning task, which is the condition SR-040 recorded for the `/2` lane and SR-047
recorded here.

## Split

FR-009 divides cleanly, and the split is the point: the resolution mechanism is
B-owned and unblocked, while only moving an entry from `unresolved` to `pinned`
waits on anyone else. Scheduling the whole requirement behind seven external
acceptances would leave the integrations with nothing to resolve against.

### B-owned, unblocked — no external delivery required

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
- [ ] Implement TC-013 against B-owned fixtures.
- [ ] Implement the per-scenario evidence record: resolution state (full or
  partial plus causing entry), per-value owner, typed outcome, per-dimension
  budget granted and consumed, trace binding.
- [ ] Implement the per-refusal-axis distinctness check as an integration-level
  invariant over retained outcomes, not a per-scenario assertion.

### Blocked — one gate per owner

- [ ] Move each entry from `unresolved` to `pinned` as its owner publishes. The
  gates and each entry's current state are recorded in FR-009's Dependencies;
  all seven are open, including A `/1`, whose live pin is branch-only.
- [ ] Execute the integration scenarios behind each pinned entry.

## Deliverables

- The version-lock manifest artifact and its resolution implementation, TC-013,
  the per-scenario evidence record, and the distinctness invariant.

## Boundary

Resolution indexes what was consumed; it never produces, canonicalizes, stores,
or assesses a producer artifact. No local evidence store, catalogue, sufficiency
decision, or human-disposition path. AD-001's qualification-evidence intake
section is normative.

Completing this task closes FR-009, NFR-005 and their scenarios and nothing
else. It establishes no PT02 result, no F observation adequacy, no replay
agreement and no campaign completion.
