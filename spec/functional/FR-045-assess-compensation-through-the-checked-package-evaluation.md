---
id: FR-045
title: "Assess compensation through the checked-package evaluation"
type: FR
relationships:
  - { target: ix://agent-ix/quire-protocol/US-001, type: implements }
  - { target: ix://agent-ix/quire-protocol/FR-003, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-024, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-025, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-044, type: depends_on }
  - { target: ix://agent-ix/quire-spec-language/FR-520, type: depends_on }
---
# FR-045: Assess compensation through the checked-package evaluation

## Description

When compensation is assessed, the library SHALL evaluate each guard, retry
relation, recovery predicate and capture through qsl-eval's
`evaluate_protocol_slot` over the strictly read checked package, SHALL map each
outcome as QSL FR-520's outcome map states, and SHALL keep the compensation state
machine (registration, ordering, attempts, retry bound as a count, recovery
decision) in this library.

## Inputs

The `CheckedPackageEvaluation` over the admitted package, a `ProtocolSlot`, a
`ProtocolSupply` built from the observation views, and a `Meter`.

## Outputs

The compensation assessment of FR-003, with each slot outcome as the successor
of the retired `graph.rs` outcome.

## Behavior

- The engine shall map `Evaluated(Completed(true))`, `Evaluated(Completed(false))`,
  `Unavailable`, `Evaluated(Undefined)`, `CallFailure::Admission`,
  `Evaluated(Refused)`, the family's unsupported result and
  `Evaluated(Incomplete)` as QSL FR-520's outcome map states.
- The engine shall never read an undefined, refused, incomplete or unavailable
  result as a Boolean (FR-024-AC-6).
- The engine shall perform no network, storage, telemetry or business-operation
  I/O (FR-025).
- The engine shall read a retry bound as the node's `attempts maximum` argument
  through `protocol_view`.
- The engine shall name no `StateView`, `EvaluationReport`, `state::evaluate_v2`
  or `ACCOUNTING_VERSION` of the retired `state` module.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-045-AC-1 | For every row of QSL FR-520's outcome map, a test over **R**'s compensations produces the successor outcome: true and false `trigger.guard`, `retry.relation` and `recovery.predicate`, an unavailable binder, a wrongly typed value, an unsupported construct and a meter one below measured usage. | Test (TC-151) |
| FR-045-AC-2 | The behaviors of FR-020-AC-1 to AC-5, FR-021, FR-022, FR-023 and FR-024-AC-1 to AC-6 asserted by `tc_003` hold with the same assertions, the report-based ones restated on the outcome map. | Test (TC-151) |
| FR-045-AC-3 | A recovery supply with an incomplete population, a missing target state, a missing achieved state or an unavailable binder never yields a true recovery, and no engine function converts an evaluation result to a Boolean (no `unwrap_or(false)`, no `as_bool`). | Test (TC-151) |
| FR-045-AC-4 | A retry bound of `n` read from `attempts maximum` admits exactly `n` attempts, counting the initial attempt, and `n + 1` is the retry-bound refusal. | Test (TC-151) |
| FR-045-AC-5 | The compensation module performs no I/O: a run under a denied-syscall harness completes, and the dependency tree of the module names no I/O crate. | Test (TC-151) |
| FR-045-AC-6 | No source file of the crate names `StateView`, `EvaluationReport`, `state::evaluate_v2` or `ACCOUNTING_VERSION` from the retired module. | Inspection (TC-151) |
| FR-045-AC-7 | Per-binder charge attribution is read from the `Meter` charge log once Contract IR provides it; until then the assessment records the call's total charge and names the missing attribution as an unsupported result, not as a zero. | Test (TC-151) |

## Dependencies

- QSL FR-520 and QSL FR-390 (`evaluate_protocol_slot`), QSL FR-519
  (`protocol_view`), FR-044.
- [FR-003](FR-003-assess-compensation-state.md) states the assessment contract;
  this requirement changes only its evaluation source.
