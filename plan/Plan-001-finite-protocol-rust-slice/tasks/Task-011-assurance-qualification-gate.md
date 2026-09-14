---
id: Task-011
title: "Complete shared assurance and qualification"
type: Task
status: blocked
track: Qualification
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-007, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-013, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-014, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-008, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-009, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-018, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/StR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-006, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-007, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-003, type: references }
  - { target: ix://agent-ix/quire-protocol/IT-003, type: verifies }
  - { target: ix://agent-ix/quire-protocol/TC-010, type: verifies }
---
# Task-011: Complete Shared Assurance and Qualification

## Scope

Own the final `qualification` adapter module. Exercise canonical/E02 results
through accepted Engineering Assurance and Quoin interfaces, then run the
complete local assurance gate on one unchanged head.

## Subtasks

- [ ] Record delivered Engineering Assurance
  `producer-execution-request/v1` and `producer-execution-result/v1` entries, accepted E02,
  the future Quoin Protocol scenario-record intake, and the shared Rust language-
  checker revision.
- [ ] Implement IT-003 SC-01 through SC-05 without a local runner, stdout verdict parser, evidence store, catalogue, sufficiency decision, or human disposition path.
- [ ] Complete TC-010 with the accepted shared Rust checker over tracked sources,
  shebangs, manifests, inline workflow commands, generated templates, runtime
  dependencies, extensionless/renamed paths, Rust-wrapper bypasses, Cargo
  metadata, workflows, unsafe code, shared boundary symbols, and bare
  `ix-trace-rs` bindings; add no local scanner.
- [ ] Run local formatting, denied-warning Clippy, tests, dependency policy, fuzz target compilation, fixture validation, and Quire validation/coverage with one Cargo job under the shared lock.
- [ ] Apply `/home/peter/dev/agent-skills/rust-review/SKILL.md` and remediate every in-scope finding on the same head.
- [ ] Run semantic QUOIN `/gap-analysis`, reconcile every task, matrix row, trace, and underspecified symbol, and close every in-scope gap.

## Deliverables

- Rust IT-003 and TC-010 evidence, exact Rust review, semantic gap-analysis artifact, complete matrix/trace reconciliation, and unchanged-head local gate record.

## Notes

- Engineering Assurance #34 is delivered. The remaining external gates are E02
  owner acceptance reconciliation (`agent-ix/quire-verification#32`) and Quoin's
  missing Protocol scenario intake (`agent-ix/quoin#519`).
- Do not trigger hosted CI; any workflow present must expose only manual dispatch.
