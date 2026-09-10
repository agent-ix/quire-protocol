---
id: FR-003
title: "Assess compensation state"
type: FR
relationships:
  - target: ix://agent-ix/quire-protocol/US-001
    type: implements
  - target: ix://agent-ix/quire-protocol/FR-002
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-055
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-058
    type: depends_on
---
# FR-003: Assess compensation state

## Description

When protocol observations establish activation, effects, commits, retries, or
recovery facts, the evaluator SHALL derive compensation state only through the
selected source requirements FR-055 through FR-058 without invoking any business action.

## Inputs

A linked protocol, exact trigger and effect observations, immutable captures and
evaluation anchors, authored retry/commit/recovery relations, and exact
progress/population/completeness authorities.

## Outputs

Typed activation, registration, attempt, effect, commit, and recovery facts with
violations, pending dependencies, or refusals.

## Behavior

The evaluator SHALL keep activation independent from truth. It SHALL preserve
trigger, capture, anchor, delivery, attempt, effect, registration, commit and
recovery identities and states. An absent trigger under complete closed input is
inactive; an unavailable trigger under open or incomplete input is
activation-unknown. The evaluator SHALL register compensation only after the
paired successful effect and SHALL NOT create another registration when that
same effect identity is replayed. It SHALL enforce the finite retry bound and
commit ordering; evaluate only the authored full or partial recovery relation
against its exact captured target and achieved state; and require exact complete
population authority for aggregated partial recovery. A successful send,
receive, delivery, attempt or operation invocation SHALL NOT establish a
successful business effect or recovery relation.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-003-AC-1 | Established, absent-under-complete-closure and unavailable-under-open/incomplete triggers yield triggered, inactive and activation-unknown states respectively; substituting the workflow, role, effect, predicate, capture or evaluation-anchor identity refuses activation. | Test (TC-003) |
| FR-003-AC-2 | One paired successful effect registers one matching compensation after the effect, while sends, receives, deliveries, failed attempts and replay of the same effect identity create no registration. | Test (TC-003) |
| FR-003-AC-3 | Compensation before its effect, registration after commit, compensation after a forbidding commit, and attempts beyond the retry bound remain distinct violations. | Test (TC-003) |
| FR-003-AC-4 | Failed, timed-out, duplicate and successful compensation attempts/effects remain distinct and retain their exact attempt, effect and registration identities. | Test (TC-003) |
| FR-003-AC-5 | An achieved partial state may satisfy an authored partial-recovery relation while failing full restoration; either success requires the exact authored relation, captured target, achieved state and applicable complete population. | Test (TC-003) |
| FR-003-AC-6 | Missing trigger, state, effect, commit, population, membership or completeness authority cannot produce activation, registration or recovery success. | Test (TC-003) |
| FR-003-AC-7 | The implementation performs no network, storage, telemetry, business-operation or compensation-operation I/O while assessing compensation. | Test (TC-003, TC-009) |

## Dependencies

- [FR-002](./FR-002-link-finite-protocol-representation.md).
- `ix://agent-ix/quire-specification/FR-055` through `FR-058`.
