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

- [ ] Write TC-002 model and mutation cases before the linker.
- [ ] Define distinct newtypes for every definition, role instance, workflow
  instance, participant, component, endpoint, channel, message, send, receive,
  delivery, attempt, and effect identity.
- [ ] Link authored causal, join, iteration, channel, and choice-visibility premises without using input order or display text.
- [ ] Accumulate independently established located diagnostics while ensuring any construction-fatal refusal returns no linked representation.
- [ ] Prove legal interleavings preserve the same authored graph; every source
  FR-050..FR-054 identity, control, termination, channel, choice and visibility
  mutation reaches its intended guard; and every authored semantic change alters
  protocol identity while presentation changes do not.

## Deliverables

- Finite representation types, linker and diagnostics, generated/model fixtures, TC-002, and trace bindings.

## Notes

- The representation is the single shared input to compensation and conformance.
- Unblocks Tasks 003 and 009.
