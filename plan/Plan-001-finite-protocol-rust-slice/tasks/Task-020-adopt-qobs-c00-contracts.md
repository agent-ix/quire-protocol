---
id: Task-020
title: "Adopt the QObs C00 consumer contract successor"
type: Task
status: done
track: Producer-Integration
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-009, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-006, type: references }
  - { target: ix://agent-ix/quire-protocol/IT-002, type: verifies }
---
# Task-020: Adopt the QObs C00 Consumer Contract Successor

## Scope

Prepare Protocol's exact QObs dependency and closed accepted-interface inventory
at C00 revision `924006300f45b38483be1cbdf99b68f899b7d368` without changing
Task-010's assessment semantics. Recognize the owner-published partial,
activation, authority-bundle v1/v2 and lineage contracts and schema digests;
report repair and aggregate-query as distinct typed unsupported Protocol
consumer capabilities because no adapters exist.

## Subtasks

- [x] Repin `quire-observation` in `Cargo.toml` and `Cargo.lock` and update the
  exact revision and rights description in `DEPENDENCIES.md`.
- [x] Extend `ProducerContract` exhaustively from the QObs owner's exported
  contract constants and schema byte arrays; author no local schema or decoder.
- [x] Bind each newly accepted schema byte array to its independently recorded
  SHA-256 and fail SC-08 when contract, bytes or digest drift.
- [x] Expose a closed Protocol-owned capability/status type in which QObs repair
  and aggregate-query are separate `unsupported` values and mint no manifest
  entries or request capabilities.
- [x] Update IT-002 SC-08 assertions for the nineteen total QObs/QSL contracts,
  exact QObs revision, five new schema digests, and the two unsupported
  consumer capabilities.
- [x] Run the scoped Rust review, scoped gap analysis, exact local gate and
  mainline PR handoff without modifying the existing dirty Protocol worktree.
  The accepted owner merge is repinned and the promotion evidence is complete.

## Deliverables

- Exact QObs C00 dependency pin, lock/rights reconciliation, owner-backed
  contract/schema inventory, typed unsupported capability surface, and IT-002
  SC-08 regression evidence.

## Notes

- This is a compatibility successor to done Task-010, not a reopening of its
  observation/temporal assessment semantics or FR-017 record composition.
- Supporting repair or aggregate query is out of scope and requires a separate
  accepted adapter specification. The presence of QObs modules at the pin is
  not support evidence.
- After Task-010 merged through PR #89, the pull request targets `main` and
  retains only Task-020's SC-08 successor delta.
- The exact pin is accepted QObs PR #27 merge
  `924006300f45b38483be1cbdf99b68f899b7d368`. Any future owner revision requires
  a repin and renewed compatibility review.
