---
id: TM-001
title: "Protocol implementation acceptance"
type: TestMatrix
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-003, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-005, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-006, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-007, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-003, type: references }
---
## Overview

This matrix plans the first Rust implementation slice. No row claims executed
coverage. The source candidate, F consumer correction, Engineering Assurance
producer, license, implementation plan, and retained corpus remain explicit gates.

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- |
| FR-001 | FR-001-AC-1, FR-001-AC-2, FR-001-AC-3 | TC-001, IT-001 | 🚧 Planned |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4 | TC-002, IT-001 | 🚧 Planned |
| FR-003 | FR-003-AC-1, FR-003-AC-2, FR-003-AC-3, FR-003-AC-4, FR-003-AC-5 | TC-003, IT-002 | 🚧 Planned |
| FR-004 | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | TC-004, IT-002 | 🚧 Planned |
| FR-005 | FR-005-AC-1, FR-005-AC-2, FR-005-AC-3, FR-005-AC-4 | TC-005 | 🚧 Planned |
| FR-006 | FR-006-AC-1, FR-006-AC-2, FR-006-AC-3, FR-006-AC-4, FR-006-AC-5, FR-006-AC-6 | TC-006, IT-002, IT-003 | 🚧 Planned |
| FR-007 | FR-007-AC-1, FR-007-AC-2, FR-007-AC-3, FR-007-AC-4, FR-007-AC-5 | TC-007, IT-003 | 🚧 Planned |
| FR-008 | FR-008-AC-1, FR-008-AC-2, FR-008-AC-3, FR-008-AC-4 | TC-011 | 🚧 Planned |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
| --- | --- | --- | --- |
| NFR-001 | Property and metamorphic testing | TC-008 | 🚧 Planned |
| NFR-002 | Property and mutation testing | TC-009 | 🚧 Planned |
| NFR-003 | Static analysis, architecture inspection, Rust review, Quire coverage, gap analysis | TC-010, IT-003 | 🚧 Planned |

## Integration Coverage

| Integration Test | Boundary | Requirements | Status |
| --- | --- | --- | --- |
| IT-001 | Native compiler artifact to protocol library | FR-001, FR-002 | 🚧 Planned |
| IT-002 | Observation and temporal facts to protocol evaluator/result | FR-003, FR-004, FR-006 | 🚧 Planned |
| IT-003 | Protocol result through Engineering Assurance, E02, and Quoin | FR-006, FR-007, NFR-003 | 🚧 Planned |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
| --- | --- | --- | --- | --- | --- |
| TC-001 | Admit only the selected protocol profile | Property | P0 | FR-001 | 🚧 Planned |
| TC-002 | Link and discriminate finite protocol structure | Property | P0 | FR-002 | 🚧 Planned |
| TC-003 | Assess compensation, retry, commit, and recovery | Property | P0 | FR-003 | 🚧 Planned |
| TC-004 | Assess finite global conformance | Property | P0 | FR-004 | 🚧 Planned |
| TC-005 | Separate protocol claim dispositions | Property | P0 | FR-005 | 🚧 Planned |
| TC-006 | Produce and strictly read canonical protocol results | Property | P0 | FR-006 | 🚧 Planned |
| TC-007 | Project canonical protocol results to E02 | Property | P0 | FR-007 | 🚧 Planned |
| TC-008 | Reproduce results under equivalent presentations | Property | P0 | NFR-001 | 🚧 Planned |
| TC-009 | Enforce every protocol resource boundary | Property | P0 | NFR-002 | 🚧 Planned |
| TC-010 | Preserve Rust and shared assurance containment | Static | P0 | NFR-003 | 🚧 Planned |
| TC-011 | Return typed refusals without text parsing | Property | P0 | FR-008 | 🚧 Planned |

## Test Execution Summary

| Category | Total | Passed | Failed | Blocked | Planned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Property | 10 | 0 | 0 | 0 | 10 |
| Static/Inspection | 1 | 0 | 0 | 0 | 1 |
| Integration | 3 | 0 | 0 | 0 | 3 |
| Overall | 14 | 0 | 0 | 0 | 14 |
