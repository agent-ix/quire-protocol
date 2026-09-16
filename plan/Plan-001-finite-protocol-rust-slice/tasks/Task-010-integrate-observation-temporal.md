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

## Deliverables

- Pinned producer contracts, Rust IT-002 harness and fixtures, adverse-path controls, and trace bindings.

## Notes

- QObs FR-287/FR-262 and authority facts are delivered; the current Protocol pin
  is merely stale. QSL supplies the native-temporal request/result owner types.
- QCI's accepted FR-025 bridge consumes Protocol leaf-result views downstream
  to construct valuations; QCI #71 then joins QSL-native and TL formula-wide
  owner results. Neither QCI output is a Protocol production dependency.
- Task-006's still-open authenticated support and correction capabilities are
  wider plan work. They do not invalidate this task's delivered SC-01 through
  SC-04 paths and are not silently credited here.
- `tc_002_linked_assessment_borrows_one_admitted_v2_protocol_context` and
  `tc_006_engine_constructs_and_strict_reads_canonical_rows` are the paired
  Rust IT-002 harness. They retain QObs/QSL owner values through FR-300,
  conformance, dispatch, and canonical result production without a local
  producer mirror.
