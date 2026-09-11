---
id: Task-010
title: "Integrate observation and temporal facts"
type: Task
status: blocked
track: B
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

Exercise accepted F/E-owned decision-scope and surrounding-execution
progress/closure, the PT02-owned complete-global-conformance closure record,
settlement, decision-support, observation, and temporal facts through
compensation, conformance, and canonical results.

## Subtasks

- [ ] Freeze exact accepted observation and temporal producer revisions with all canonical-result input dimensions.
- [ ] Implement IT-002 SC-01 through SC-04 as Rust integration tests.
- [ ] Preserve producer identities and authorities without deriving missing facts or copying producer semantics.
- [ ] Exercise decisive and non-decisive open prefixes, missing support,
  independently changed or cross-wired decision-scope/surrounding-execution
  progress and closure, every global closure state and required execution,
  branch, and workflow premise mutation, mis-correlation, duplication, and late
  contradiction.
- [ ] Prove every adverse case yields its exact refusal, violation, incompleteness, or immutable supersession outcome.

## Deliverables

- Pinned producer contracts, Rust IT-002 harness and fixtures, adverse-path controls, and trace bindings.

## Notes

- F's complete canonical-result consumer shape is present in Quire Specification
  PR #15 at `d8d984b542fe30f65b6e07f4a07b2d1f81d3962b`, including all five
  progress/closure axes and premise mutations. It remains an unaccepted input
  until the composed baseline is reviewed and promoted.
- Blocked by Task-006, composed acceptance of that F consumer, and the accepted
  temporal profile.
