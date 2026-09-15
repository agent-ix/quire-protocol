---
id: Task-010
title: "Integrate observation and temporal facts"
type: Task
status: done
track: Producer-Integration
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-006, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-003, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-006, type: references }
  - { target: ix://agent-ix/quire-protocol/IT-002, type: verifies }
---
# Task-010: Integrate Observation and Temporal Facts

## Scope

Exercise accepted QObs decision-scope and surrounding-execution authority facts,
the Protocol-owned complete-global-conformance closure record, and accepted QSL
native-temporal results through compensation, conformance and canonical results.
The integration also consumes QSL FR-053's versioned opaque-trigger v2 bridge
before it derives any event-triggered FR-300 obligation binding.

## Subtasks

- [x] Repin QObs to accepted `93bc565` and resolve only the scenario-used
  `record/v2`, `population/v2`, `explicit-members/v1`, `position-ledger/v1`,
  `clock-binding/v1`, `capture-environment/v1`, `progress-assertion/v1`,
  `closure-assertion/v1`, `completeness-assertion/v1` and
  `result-availability/v1` owner contracts; resolve QSL
  `quire.native-temporal-request/v1` and `quire.native-temporal-result/v1` at the
  accepted compiler pin, then adopt the FR-053 opaque-trigger v2 contracts when
  their producer implementation is available.
- [x] Implement IT-002 SC-01 through SC-04 as Rust integration tests.
- [x] Preserve producer identities and authorities without deriving missing facts or copying producer semantics.
- [x] Exercise decisive and non-decisive open prefixes, missing support,
  independently changed or cross-wired decision-scope/surrounding-execution
  progress and closure, every global closure state and required execution,
  branch, and workflow premise mutation, mis-correlation, duplication, and late
  contradiction.
- [x] Prove every adverse case yields its exact refusal, violation, incompleteness, or immutable supersession outcome.
- [x] Execute SC-05a through SC-05d batch/incremental correspondence, early
  settlement, non-singleton permitted arrival permutation and changed-premise
  controls; execute SC-06 exact/one-short/fresh request bounds and SC-08 exact
  producer-resolution controls.
- [x] Retain SC-07a's Task-010-owned inputs — the full resolved manifest,
  selected producer contracts, owner provenance, granted/consumed budgets and
  typed outcomes — on the completed bundle. Task-018 remains the sole owner of
  the final FR-017 record and SC-07b record-set distinctness validation.

## Deliverables

- Pinned producer contracts, Rust IT-002 harness and fixtures, adverse-path
  controls, trace bindings, and exact resolution/resource attribution inputs
  for Task-018's final evidence composer.

## Notes

- QObs FR-287/FR-262 and authority facts are delivered; the current Protocol pin
  is merely stale. QSL supplies the native-temporal request/result owner types.
- QCI's accepted FR-025 bridge consumes Protocol leaf-result views downstream
  to construct valuations; QCI #71 then joins QSL-native and TL formula-wide
  owner results. Neither QCI output is a Protocol production dependency.
- The remaining production prerequisite is Task-006, not an F-owned result mapper
  or PT02 producer.
- Completed assessment bundles expose only Task-018's typed composition inputs;
  this task does not define, store or hand off the final scenario evidence
  record and does not claim SC-07b complete.
