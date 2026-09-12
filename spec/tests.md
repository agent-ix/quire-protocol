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
  - { target: ix://agent-ix/quire-protocol/NFR-004, type: references }
---
## Overview

This matrix plans the first Rust implementation slice. No row claims executed
coverage. The accepted frozen source baseline, F consumer correction, Engineering
Assurance producer, license, implementation plan, and retained corpus remain
explicit gates.

Task-009 has a separately invoked producer-fixture smoke lane for its first
consumer increment. Its tests remain `#[ignore]` because they require A's
stripped release producer output and retain A's draft wire labels; they do not
change the planned status of FR-001, FR-002, or IT-001's full scenario matrix.

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- |
| FR-001 | FR-001-AC-1, FR-001-AC-2, FR-001-AC-3, FR-001-AC-4 | TC-001, IT-001 | 🚧 Planned |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4 | TC-002, IT-001 | 🚧 Planned |
| FR-003 | FR-003-AC-1, FR-003-AC-2, FR-003-AC-3, FR-003-AC-4, FR-003-AC-5, FR-003-AC-6, FR-003-AC-7 | TC-003, IT-002 | 🚧 Planned |
| FR-004 | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | TC-004, IT-002 | 🚧 Planned (FR-004-AC-3 landed by `tests/tc_004_closure_axes.rs`) |
| FR-005 | FR-005-AC-1, FR-005-AC-2, FR-005-AC-3, FR-005-AC-4, FR-005-AC-5 | TC-005 | 🚧 Planned (FR-005-AC-1 disposition shape landed by `tests/tc_005_claim_dispositions.rs`; its end-to-end scenario and AC-2..AC-5 pending) |
| FR-006 | FR-006-AC-1, FR-006-AC-2, FR-006-AC-3, FR-006-AC-4, FR-006-AC-5, FR-006-AC-6, FR-006-AC-7 | TC-006, IT-002, IT-003 | 🚧 Planned |
| FR-007 | FR-007-AC-1, FR-007-AC-2, FR-007-AC-3, FR-007-AC-4, FR-007-AC-5 | TC-007, IT-003 | 🚧 Planned |
| FR-008 | FR-008-AC-1, FR-008-AC-2, FR-008-AC-3, FR-008-AC-4, FR-008-AC-5, FR-008-AC-6, FR-008-AC-7, FR-008-AC-8 | TC-011 | 🚧 Planned (FR-008-AC-1, AC-2 and AC-7 landed by `tests/tc_011_typed_refusals.rs`; the structural halves of AC-4 and AC-8 landed there too) |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
| --- | --- | --- | --- |
| NFR-001 | Property and metamorphic testing | TC-008 | 🚧 Planned |
| NFR-002 | Property, mutation, and source-bound inventory inspection | TC-009 | 🚧 Planned |
| NFR-003 | Shared Rust language-boundary check, static analysis, architecture inspection, Rust review, Quire coverage, gap analysis | TC-010, IT-003 | 🚧 Planned |
| NFR-004 | Cargo metadata inspection, conditional-compilation inventory, enumerated gate execution over one unchanged head, per-criterion negative controls | TC-012, IT-003 | 🚧 Planned |

## Integration Coverage

| Integration Test | Boundary | Requirements | Status |
| --- | --- | --- | --- |
| IT-001 | Native compiler artifact to protocol library | FR-001, FR-002 | 🚧 Planned |
| IT-002 | Observation and temporal facts to protocol evaluator/result | FR-003, FR-004, FR-006 | 🚧 Planned |
| IT-003 | Protocol result through Engineering Assurance, E02, and Quoin | FR-006, FR-007, NFR-003, NFR-004 | 🚧 Planned |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
| --- | --- | --- | --- | --- | --- |
| TC-001 | Admit only the selected protocol profile | Property | P0 | FR-001 | 🚧 Planned |
| TC-002 | Link and discriminate finite protocol structure | Property | P0 | FR-002 | 🚧 Planned (FR-002-AC-1 identity set executing; AC-2..AC-4 unwritten) |
| TC-003 | Assess compensation, retry, commit, and recovery | Property | P0 | FR-003 | 🚧 Planned |
| TC-004 | Assess finite global conformance | Property | P0 | FR-004 | 🚧 Planned (FR-004-AC-3 landed; AC-1/AC-2/AC-4 pending the batch and incremental evaluators) |
| TC-005 | Separate protocol claim dispositions | Property | P0 | FR-005 | 🚧 Planned (six-disposition record and its no-aggregate-verdict proof landed; premise retention, dispatch and the conformance controls pending) |
| TC-006 | Produce and strictly read canonical protocol results | Property | P0 | FR-006 | 🚧 Planned |
| TC-007 | Project canonical protocol results to E02 | Property | P0 | FR-007 | 🚧 Planned |
| TC-008 | Reproduce results under equivalent presentations | Property | P0 | NFR-001-AC-1..NFR-001-AC-3 | 🚧 Planned — reproduction-record slice executed (12 tests, chunk C-31); equivalent-presentation permutations and the full one-axis mutation matrix remain planned |
| TC-009 | Enforce every protocol resource boundary | Property | P0 | NFR-002-AC-1..NFR-002-AC-4 | 🚧 Planned |
| TC-010 | Preserve Rust and shared assurance containment | Static | P0 | NFR-003-AC-1..NFR-003-AC-6 | 🚧 Planned |
| TC-011 | Return typed refusals without text parsing | Property | P0 | FR-008-AC-1..FR-008-AC-8 | 🚧 Planned (envelope, code catalog and repo-wide code audit landed; AC-3/AC-6 pending catalog selection, AC-5 and the policy halves of AC-4/AC-8 pending the cause taxonomy) |
| TC-012 | Run the local verification gate and refuse an undefined lane | Static | P0 | NFR-004-AC-1..NFR-004-AC-10 | 🚧 Planned |

## Test Execution Summary

| Category | Total | Passed | Failed | Blocked | Planned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Property | 10 | 0 | 0 | 0 | 10 |
| Static/Inspection | 2 | 0 | 0 | 0 | 2 |
| Integration | 3 | 0 | 0 | 0 | 3 |
| Overall | 15 | 0 | 0 | 0 | 15 |
