---
id: Task-003
title: "Assess compensation state"
type: Task
status: in_progress
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

- [ ] Write TC-003 activation, state-transition, relation, idempotence, and
  adverse-path cases first.
- [ ] Derive triggered, inactive and activation-unknown only from exact
  QSpec FR-300 activation bindings and the applicable progress, closure and
  completeness authority; refuse any caller-built or cross-linked substitute.
- [ ] Preserve delivery, attempt, effect, registration, commit, and recovery identities as distinct types.
- [ ] Register compensation only after the paired successful effect, preserve one
  registration when that exact effect is replayed, and enforce authored finite
  retry and commit ordering.
- [ ] Evaluate full and partial recovery only through their exact authored
  relation, captured target and achieved state; require exact population,
  membership and completeness authority for aggregate partial recovery.
- [ ] Remove the public `RecoveryRelation` callback from production. Evaluate
  only the admitted QSL typed state/temporal expression under its resolved
  capability and request accountant; unknown/out-of-profile expressions are
  unsupported, and source refusal/resource incompleteness propagate exactly.
- [ ] Prove with zero-call action-capable doubles and API/dependency mutations
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
