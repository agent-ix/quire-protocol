---
id: Task-011
title: "Complete shared assurance and qualification"
type: Task
status: blocked
track: Gate
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-008, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-009, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/Task-010, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/StR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-006, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-007, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-003, type: references }
  - { target: ix://agent-ix/quire-protocol/IT-003, type: verifies }
  - { target: ix://agent-ix/quire-protocol/TC-010, type: verifies }
---
# Task-011: Complete Shared Assurance and Qualification

## Scope

Exercise canonical/E02 results through accepted Engineering Assurance and Quoin
interfaces, then run the complete local assurance gate on one unchanged head.

## Subtasks

- [ ] Freeze the accepted Engineering Assurance issue 34, E02, Quoin interface,
  Quoin containment (`quire-research#62`), and shared Rust language-checker
  (`quire-research#64`) revisions.
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

- Blocked by Tasks 008 through 010 and the accepted external interfaces.
- Do not trigger hosted CI; any workflow present must expose only manual dispatch.
