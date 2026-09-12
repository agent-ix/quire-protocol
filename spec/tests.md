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
  - { target: ix://agent-ix/quire-protocol/FR-009, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-005, type: references }
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

The nine object requirements FR-010..FR-018 are counted in the Domain Object
Coverage table, not in the test-case totals: they are specified nouns, and the
test cases that exercise them are already counted above.

Every protocol, temporal and observation noun this repository's requirements use
now resolves to an `ix://agent-ix/quire-specification/FR-2NN` reference rather
than a local definition. Where an upstream artifact records an open gap, the
dependent requirement cites the gap rather than supplying the missing identity,
key or order. Those citations are why several criteria below stand as obligations
that cannot yet be discharged — see `quire-specification` issues 28 and 29.

IT-001 is split into two rows because `/1` and `/2` are separate contracts that
may not substitute for each other. The `/2` row is planned and unexecuted. A's `protocol_artifact::v2` is
implemented and under final integration; its public surface is recorded in
IT-001 Preconditions and B builds against those exact names, but the surface is
not yet on `quire-spec-language` `origin/main` or any remote branch, so B has
not verified it directly and pins no intermediate commit. SC-06 through SC-09
execute when A publishes its one final immutable revision. The
B-owned tests that exist for `/2` (`tests/it001_v2_intake.rs`) are tagged
`Task-009` only and carry no `IT-001` trace tag: they assert B's own evidence
loader and scenario corpus and make no claim about A's behaviour, so they must
not be counted as backing symbols for any IT-001 scenario.

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- |
| FR-001 | FR-001-AC-1, FR-001-AC-2, FR-001-AC-3, FR-001-AC-4, FR-001-AC-5 | TC-001, IT-001 | 🚧 Planned (FR-001-AC-5 gated on A's final `/2` revision) |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-5 | TC-002, IT-001 | 🚧 Planned (FR-002-AC-5 gated on A's final `/2` revision) |
| FR-003 | FR-003-AC-1, FR-003-AC-2, FR-003-AC-3, FR-003-AC-4, FR-003-AC-5, FR-003-AC-6, FR-003-AC-7 | TC-003, IT-002 | 🚧 Planned |
| FR-004 | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | TC-004, IT-002 | 🚧 Planned (FR-004-AC-3 landed by `tests/tc_004_closure_axes.rs`) |
| FR-005 | FR-005-AC-1, FR-005-AC-2, FR-005-AC-3, FR-005-AC-4, FR-005-AC-5 | TC-005 | 🚧 Planned |
| FR-006 | FR-006-AC-1, FR-006-AC-2, FR-006-AC-3, FR-006-AC-4, FR-006-AC-5, FR-006-AC-6, FR-006-AC-7 | TC-006, IT-002, IT-003 | 🚧 Planned |
| FR-007 | FR-007-AC-1, FR-007-AC-2, FR-007-AC-3, FR-007-AC-4, FR-007-AC-5 | TC-007, IT-003 | 🚧 Planned |
| FR-008 | FR-008-AC-1, FR-008-AC-2, FR-008-AC-3, FR-008-AC-4, FR-008-AC-5, FR-008-AC-6, FR-008-AC-7, FR-008-AC-8 | TC-011 | 🚧 Planned |
| FR-009 | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5, FR-009-AC-6, FR-009-AC-7 | TC-013, IT-002, IT-003 | 🚧 Planned — TC-013 exercises resolution on B-owned fixtures with no external dependency; the IT rows stay gated on producer acceptance |

## Domain Object Coverage

Requirements carrying `object:` specify the consumer-boundary vocabulary. They
are separated from the behavioural table because they define nouns rather than
behaviour.

Sixty-six criteria across the nine. Twenty-one verify by inspection or analysis;
the rest name a test case, and each named test case declares the object criteria
it backs in its own Traceability section, so the reference resolves in both
directions.

| Object Req | Object type | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- | --- |
| FR-010 | domain | FR-010-AC-1..AC-3 | — (Inspection) | 🚧 Planned |
| FR-011 | value_object | FR-011-AC-1..AC-7 | TC-014 | 🚧 Planned |
| FR-012 | entity | FR-012-AC-1..AC-8 | TC-013 | 🚧 Planned |
| FR-013 | value_object | FR-013-AC-1..AC-8 | TC-001, TC-014 | 🚧 Planned |
| FR-014 | value_object | FR-014-AC-1..AC-8 | TC-011, TC-014 | 🚧 Planned |
| FR-015 | enumeration | FR-015-AC-1..AC-7 | TC-001, TC-014 | 🚧 Planned — the production and `/2` digest enums are two different vocabularies sharing one member name; FR-015 records the divergence as unreconciled |
| FR-016 | enumeration | FR-016-AC-1..AC-8 | TC-008, TC-009, TC-011, TC-013 | 🚧 Planned |
| FR-017 | entity | FR-017-AC-1..AC-8 | TC-008, TC-009, TC-010, TC-013 | 🚧 Planned |
| FR-018 | entity | FR-018-AC-1..AC-9 | TC-013 | 🚧 Planned |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
| --- | --- | --- | --- |
| NFR-001 | Property and metamorphic testing | TC-008 | 🚧 Planned |
| NFR-002 | Property, mutation, and source-bound inventory inspection | TC-009, IT-001 | 🚧 Planned (IT-001 SC-07/SC-09 gated on A's final `/2` revision) |
| NFR-003 | Shared Rust language-boundary check, static analysis, architecture inspection, Rust review, Quire coverage, gap analysis | TC-010, IT-003 | 🚧 Planned |
| NFR-004 | Cargo metadata inspection, conditional-compilation inventory, enumerated gate execution over one unchanged head, per-criterion negative controls | TC-012 | 🚧 Planned — IT-003 previously appeared here but declares and exercises no NFR-004 scenario |
| NFR-005 | Static inspection and Quire coverage over the integration lanes, denominated on owed rather than executed scenarios | IT-002, IT-003 | 🚧 Planned — scores zero while both integrations are blocked, by design |

## Integration Coverage

| Integration Test | Boundary | Requirements | Status |
| --- | --- | --- | --- |
| IT-001 `/1` | Native compiler `quire.compiled-protocol/1` artifact to protocol library | FR-001, FR-002 | 🚧 Planned (SC-01..SC-05; fixture lane `#[ignore]`) |
| IT-001 `/2` | Native compiler `/2` artifact to protocol library | FR-001-AC-5, FR-002-AC-5, NFR-002 | 🚧 Planned — SC-06..SC-09 unexecuted; A's `protocol_artifact::v2` is implemented and under final integration, not yet published to a remote |
| IT-002 | Observation and temporal facts to protocol evaluator/result | FR-003, FR-004, FR-006, FR-009, NFR-002, NFR-005 | 🚧 Planned — SC-01..SC-08; blocked on accepted F/E producer contracts |
| IT-003 | Protocol result through Engineering Assurance, E02, and Quoin | FR-006, FR-007, FR-009, NFR-002, NFR-003, NFR-005 | 🚧 Planned — SC-01..SC-09; blocked on Engineering Assurance #34 |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
| --- | --- | --- | --- | --- | --- |
| TC-001 | Admit only the selected protocol profile | Property | P0 | FR-001 | 🚧 Planned |
| TC-002 | Link and discriminate finite protocol structure | Property | P0 | FR-002 | 🚧 Planned (FR-002-AC-1 identity set executing; AC-2..AC-4 unwritten) |
| TC-003 | Assess compensation, retry, commit, and recovery | Property | P0 | FR-003 | 🚧 Planned |
| TC-004 | Assess finite global conformance | Property | P0 | FR-004 | 🚧 Planned (FR-004-AC-3 landed; AC-1/AC-2/AC-4 pending the batch and incremental evaluators) |
| TC-005 | Separate protocol claim dispositions | Property | P0 | FR-005 | 🚧 Planned |
| TC-006 | Produce and strictly read canonical protocol results | Property | P0 | FR-006 | 🚧 Planned |
| TC-007 | Project canonical protocol results to E02 | Property | P0 | FR-007 | 🚧 Planned |
| TC-008 | Reproduce results under equivalent presentations | Property | P0 | NFR-001-AC-1..NFR-001-AC-3 | 🚧 Planned — reproduction-record slice executed (12 tests, chunk C-31); equivalent-presentation permutations and the full one-axis mutation matrix remain planned |
| TC-009 | Enforce every protocol resource boundary | Property | P0 | NFR-002-AC-1..NFR-002-AC-4 | 🚧 Planned |
| TC-010 | Preserve Rust and shared assurance containment | Static | P0 | NFR-003-AC-1..NFR-003-AC-6 | 🚧 Planned |
| TC-011 | Return typed refusals without text parsing | Property | P0 | FR-008-AC-1..FR-008-AC-8 | 🚧 Planned |
| TC-012 | Run the local verification gate and refuse an undefined lane | Static | P0 | NFR-004-AC-1..NFR-004-AC-10 | 🚧 Planned |
| TC-013 | Resolve the version-lock manifest | Property | P0 | FR-009-AC-1..FR-009-AC-7 | 🚧 Planned — B-owned fixtures only; runnable without any external producer delivery |
| TC-014 | Apply the consumer identity discipline | Property | P0 | FR-011-AC-1..FR-011-AC-7 | 🚧 Planned — B-owned fixtures only; runnable without any external producer delivery |

## Test Execution Summary

| Category | Total | Passed | Failed | Blocked | Planned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Property | 12 | 0 | 0 | 0 | 12 |
| Static/Inspection | 2 | 0 | 0 | 0 | 2 |
| Integration | 4 | 0 | 0 | 0 | 4 |
| Overall | 18 | 0 | 0 | 0 | 18 |

The Integration count is four because IT-001 carries a separate `/1` and `/2`
row; the two contracts may not substitute for each other, so one row cannot
report both.
