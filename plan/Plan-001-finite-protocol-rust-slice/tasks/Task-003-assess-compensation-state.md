---
id: Task-003
title: "Assess compensation state"
type: Task
status: complete
track: Core
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-002, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-003, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-003, type: verifies }
---
# Task-003: Assess Compensation State

## Scope

Implement pure activation, trigger/capture/anchor binding, effect,
replay-idempotent registration, retry, commit, and full/partial recovery
assessment over the linked protocol and accepted producer facts.

The replay, activation, retry, commit and recovery kernels already exist. This
task closes their missing graph/runtime-input bridge and removes independent
production construction of compensation identities.

## Subtasks

- [x] Write TC-003 activation, state-transition, relation, idempotence, and
  adverse-path cases first.
- [x] Derive triggered, inactive and activation-unknown only from exact
  QSpec FR-300 activation bindings and the applicable progress, closure and
  completeness authority; refuse any caller-built or cross-linked substitute.
- [x] Preserve delivery, attempt, effect, registration, commit, and recovery identities as distinct types.
- [x] Register compensation only after the paired successful effect, preserve one
  registration when that exact effect is replayed, and enforce authored finite
  retry and commit ordering.
- [x] Evaluate full and partial recovery only through their exact authored
  relation, captured target and achieved state; require exact population,
  membership and completeness authority for aggregate partial recovery.
- [x] Remove the public `RecoveryRelation` callback from production. Evaluate
  only the admitted QSL typed state/temporal expression under its resolved
  capability and request accountant; unknown/out-of-profile expressions are
  unsupported, and source refusal/resource incompleteness propagate exactly.
- [x] Prove with zero-call action-capable doubles and API/dependency mutations
  that the module admits no network, storage, telemetry, business-operation, or
  compensation-operation I/O and derives no fact from successful transport alone.

## Deliverables

- Pure compensation evaluator, typed outcome facts, TC-003, mutation controls, and trace bindings.

## Notes

- Unblocks Task-004.
- The accepted QObs pin `93bc565` publishes the complete authority-qualified
  FR-287 runtime subject key. The `/2` linked context projects it for
  compensation only when the corresponding QSL requirement is exactly
  `BindingKind::Effect`; QProtocol neither reconstructs nor locally mints a
  runtime effect identity. The remaining bridge derives compensation records
  from QSL's typed compensation table and these admitted owner values.
- QSpec FR-236 is accepted on QSpec main at `478afa2`: `failed`, `timed-out`,
  `pending`, and `successful` are separately admissible for per-attempt and
  recovery-relation facts; `duplicate` is effect-only. Retain these domains as
  distinct types and never infer a recovery outcome from an attempt outcome.
- QSL #101/#105/#108 are delivered through QSL PR #109 at exact merge
  `c0deb7b3ab66d6e8cb64a0fd7769bd0af82b61b7`. The production Engine now walks
  those admitted guard, retry and recovery handles directly, retains complete
  owner evaluation reports, and exposes no recovery callback in production.
- `tests/tc_003_graph_compensation.rs` exercises exact activation, finite retry,
  selected commit, QObs position-ledger ordering, full-versus-partial recovery,
  replay, missing-source and resource-exhaustion transactionality. The earlier
  pure kernels remain as internal building blocks and their action-capability
  mutation test continues to prove zero business/compensation calls.
- The graph snapshot retains the exact QSL state views used for successful
  guard, retry and recovery evaluation, the complete owner reports, each
  attempt's independently established related effect when present, and the
  complete QObs observation/position views used for causal ordering. Replay
  compares these inputs as well as their derived outcomes; changed evidence
  cannot pass as an exact replay.
