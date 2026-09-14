---
id: Task-002
title: "Link the finite protocol representation"
type: Task
status: in_progress
track: A
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-001, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-002, type: verifies }
---
# Task-002: Link the Finite Protocol Representation

## Scope

Implement the finite identity-preserving graph and all construction-fatal
validation from the admitted selected profile.

## Subtasks

- [x] Write the original TC-002 model and static mutation cases before the linker.
- [x] Define distinct newtypes for every definition, role instance, workflow
  instance, participant, component, endpoint, channel, message, send, receive,
  delivery, attempt, and effect identity.
- [x] Link the admitted artifact's causal, join, iteration, channel, and
  choice-visibility premises and accumulate independent located diagnostics.
- [x] Prove legal interleavings preserve the admitted graph and exercise the
  original FR-050..FR-054 static mutation sweep.
- [ ] Pin the accepted Observation PR #20 boundary and implement one
  version-explicit binding subsystem: an accepted static binding definition
  owns role internality and independently selected slot compatibility; a
  separate runtime set imports FR-287 subjects and FR-262 relationships. Refuse
  old, missing, surplus, duplicate, wrong-kind, incompatible and slot-derived
  bindings without defaults or universal producer-kind spellings.
- [ ] Replace the partial declaration/index identity with a structured FR-288
  definition identity derived from the admitted version-2 package's exact source
  document, declaration handle, formal owner/clause and semantic selection
  closure.
- [ ] Replace reduced control shapes and numeric fingerprints with complete
  typed declaration plus accepted static-binding consistency content; refuse
  same-FR-288/different-content through the linker registry while excluding
  concrete runtime values from FR-288.
- [ ] Retain all five FR-216 channel premises, including the complete typed FIFO
  expression definition, and expose an exact generic runtime partition over
  static channel plus owner-typed evaluated value.
- [ ] Adopt FR-262 replay and contradiction semantics without converting a
  relationship into a subject or collapsing a second exact key.
- [ ] Extend TC-002 so every newly accepted identity/content/binding axis has a
  positive control and a failing one-axis mutation; every source
  FR-050..FR-054 identity, control, termination, channel, choice and visibility
  mutation reaches its intended guard; and every authored semantic change alters
  protocol identity while presentation changes do not.
- [ ] Bind every claim with bare compiler-checked `ix-trace-rs`, run the scoped
  local gates, reconcile Plan-001/TM-001/issues #4 and #14, and close #4 with its
  reviewed PR.

## Deliverables

- Structured definition/semantic-content types, versioned runtime binding and
  relationship registry, exact channel/FIFO representation, linker diagnostics,
  generated/model fixtures, TC-002, and trace bindings.

## Notes

- The representation is the single shared input to compensation and conformance.
- Quire Specification PRs #41/#42 and Quire Observation PR #20 are delivered
  dependencies. The stale six-premise wording in Spec Language #85 is not a
  dependency: accepted FR-216 defines five components and derives duplication
  permission from delivery cardinality.
- Unblocks Tasks 003 and 009.
