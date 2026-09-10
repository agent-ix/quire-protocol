---
id: Task-008
title: "Enforce reproduction and resource bounds"
type: Task
status: blocked
track: A
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-007, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/NFR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-008, type: verifies }
  - { target: ix://agent-ix/quire-protocol/TC-009, type: verifies }
---
# Task-008: Enforce Reproduction and Resource Bounds

## Scope

Apply semantic reproduction and every caller-supplied resource guard across the
complete admission, linking, evaluation, result, and projection path.

## Subtasks

- [ ] Write TC-008 equivalent-presentation and semantic-discrimination properties across all public operations.
- [ ] Write TC-009 below/at/above cases for input, graph, branching, iteration, observation, lineage, output, operation, allocation, and deadline limits.
- [ ] Separate prechecked count/accounted-allocation refusal from monotonic deadline observation and report measured overshoot honestly.
- [ ] Kill every isolated guard-removal and identity-canonicalization mutant for the intended reason.
- [ ] Record deterministic seeds, toolchain identities, and corpus identities needed for local reproduction.

## Deliverables

- Cross-cutting resource guard, reproduction suite, mutation ratchets, TC-008, TC-009, and trace bindings.

## Notes

- Whole-process containment remains an Engineering Assurance responsibility.
- Unblocks the final assurance gate.
