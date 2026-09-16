---
id: Task-007
title: "Project canonical results to E02"
type: Task
status: done
track: Producer-Integration
priority: P0
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-006, type: depends_on }
  - { target: ix://agent-ix/quire-protocol/FR-007, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-007, type: verifies }
---
# Task-007: Project Canonical Results to E02

## Scope

Implement the selected FR-062 total projection into the exact E02 strict-reader
contract, preserving the canonical source and every mapped finding.

## Subtasks

- [x] Write TC-007 for every source row, simultaneous finding set, and refusal condition first.
- [x] Pin source, target, adapter, mapping, and dependency versions and compute the derived identity with RFC 8785.
- [x] Preserve the exact canonical result reference, oracle lineage, native artifacts, and limitations.
- [x] Keep protocol adequacy distinct from E02 adequacy and any later sufficiency disposition.
- [x] Invoke the E02 strict reader and refuse finding loss, unknown versions, or any non-total mapping without emitting a `TechniqueResult`.

## Deliverables

- E02 projection adapter, version pins, strict-reader integration, TC-007, and trace bindings.

## Notes

- Unblocks Task-008 and the E02 half of Task-011.
- qverify issue #32 accepted the draft-2 schema/reader contract at
  `d2fc0d8134c776d2f1a17fd249f016b18a6ba124`; issue #44 published the exact
  dependency-compatible consumer revision
  `293e342c6f5e2c69738956ff5165f75761b0364a`. Protocol pins that revision and
  its transitive Engineering Assurance revision
  `9237ce50ce19c6d6c937705bfde4b64adcc9d16b`, consumes the owner types, JCS
  implementation and strict reader, and implements no substitute.
