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

A linked protocol, exact observation identities, immutable captures, authored
retry/commit/recovery relations, and progress/completeness authorities.

## Outputs

Typed activation, registration, attempt, effect, commit, and recovery facts with
violations, pending dependencies, or refusals.

## Behavior

The evaluator SHALL register compensation only after the paired successful
effect. It SHALL preserve delivery, attempt, and effect identities; enforce the
finite retry bound and commit ordering; evaluate only the authored recovery
relation; and require exact complete population authority for aggregated partial
recovery. A successful operation invocation SHALL NOT establish a successful
business effect or recovery relation.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-003-AC-1 | One successful effect registers one matching compensation, while failed attempts and duplicate deliveries register none. | Test (TC-003) |
| FR-003-AC-2 | Compensation before its effect, registration after commit, compensation after a forbidding commit, and attempts beyond the retry bound remain distinct violations. | Test (TC-003) |
| FR-003-AC-3 | Failed, timed-out, duplicate, successful, partial, and incomplete recovery observations remain distinct and preserve exact identities. | Test (TC-003) |
| FR-003-AC-4 | Missing state, effect, commit, population, or completeness authority cannot produce registration or recovery success. | Test (TC-003) |
| FR-003-AC-5 | The implementation performs no network, storage, telemetry, or business-operation I/O while assessing compensation. | Analysis (TC-003) |

## Dependencies

- [FR-002](./FR-002-link-finite-protocol-representation.md).
- `ix://agent-ix/quire-specification/FR-055` through `FR-058`.
