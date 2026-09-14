---
id: Task-014
title: "Execute the local gate policy"
type: Task
status: done
track: Gate-Enablement
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/NFR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-012, type: verifies }
---
# Task-014: Execute the Local Gate Policy

## Scope

Make NFR-004 and TC-012 executable against the current repository. Validate the
one Cargo-feature lane, separately account for test-only and admitted-target
conditions, and bind promotion evidence to the exact ordered commands, clean
tracked state, commit and worktree-owned target directory.

## Subtasks

- [x] Add failing Rust TC-012 tests first for the current repository snapshot
  and for each independently invalid command, warning, feature, conditional,
  target-directory and state-binding axis.
- [x] Implement the smallest test-owned gate-policy model needed to classify
  those inputs; invoke no subprocess, stdout parser, shell wrapper, generic
  assurance runner or hosted workflow.
- [x] Update `AGENTS.md` and `README.md` so their commands and configuration
  claims agree with NFR-004 and the implemented tree.
- [x] Reconcile TM-001, lifecycle counts, this task and the plan index/log
  against the executing tests; issue #31 and epic #14 update at promotion.
- [x] Run the exact local gate on one committed clean head and record the
  default/no-default/all-features metadata identity plus each manual compiler
  negative control.
- [x] Run self `/rust-review` and `/gap-analysis` and fix every finding. Promote
  only through the one-ticket PR that closes #31, then record its merge on the
  parent epic.

## Deliverables

- Rust `tests/tc_012_local_gate.rs`, corrected contributor instructions, the
  reconciled matrix/plan, and same-head local gate/review evidence.

## Notes

- This task is independent enablement and unblocks the gate mechanics only.
  Task-011 remains blocked on its own producer, result and assurance work.
- The admitted evidence target is `x86_64-unknown-linux-gnu`. No other-platform
  support is inferred from a test that did not execute there.
- Agent B's TL-* lane and Agent E's Contract-IR bridge are out of scope.
