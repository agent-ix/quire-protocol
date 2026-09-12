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
  - { target: ix://agent-ix/quire-protocol/FR-009, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-010, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-011, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-012, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-013, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-014, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-015, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-016, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-017, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-018, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-019, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-020, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-021, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-022, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-023, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-024, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-025, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-026, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-027, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-028, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-029, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-030, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-031, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-032, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-033, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-034, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-035, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-036, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-037, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-038, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-039, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-040, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-041, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-003, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-005, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-006, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-007, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-008, type: references }
---
## Overview

This matrix plans the first Rust implementation slice. No row claims executed
coverage. The accepted frozen source baseline, F consumer correction, Engineering
Assurance producer, license, implementation plan, and retained corpus remain
explicit gates.

The repository carries 41 functional requirements, 8 non-functional
requirements, 3 integration tests and 14 test cases, and 288 acceptance criteria
across the 41 FRs and the 8 NFRs. The functional set divides into 9 aggregate or
behavioural parents (FR-001..FR-009), 9 object requirements (FR-010..FR-018),
and 23 atomic behavioural parts (FR-019..FR-041) that the parents aggregate
through `contains`. FR-003 through FR-008 now state their obligations once each
through those parts; the parent row and the part rows are therefore two views of
one obligation and must not be added together as separate work.

The 288 counts rows of a `## Acceptance Criteria` table on an FR or an NFR, and
nothing else. `quire properties` reports 308 criteria over the same tree: the
extra 20 are the `StR-0NN-VC-n` validation criteria of StR-001 through StR-005,
which are stakeholder validation rather than FR/NFR acceptance and which this
matrix carries no table for. Every one of the 288 appears in the engine's 308.
NFR-005's five metrics are outside both figures, because that requirement states
metrics and no acceptance criteria.

### What each status word in this matrix means

The four words below are not interchangeable, and no total mixes them.

- **Test-backed** — the criterion's `Verification` cell names at least one
  authored test case (`TC-0NN`). 229 of 288 criteria.
- **Integration-backed** — the cell names only an integration scenario
  (`IT-0NN`) and no test case. 12 of 288 criteria.
- **Unbacked** — the cell names no test case and no integration scenario: it
  verifies by `Inspection` or `Analysis` only. 47 of 288 criteria (37
  Inspection, 10 Analysis). An unbacked criterion is not covered by a test and
  this matrix writes `none` in its Test Cases cell rather than a test-case id
  that would imply otherwise. TC-014 is the highest authored test case; no row
  cites a higher id.
- **Unexecutable** — a backing exists but cannot be run, so the criterion is
  neither passing nor failing. 12 of 288 criteria, all of them
  integration-backed. Under `FR-009-AC-3` such a scenario is reported
  unexecutable and counted as zero, never as passing and never omitted from a
  denominator. `NFR-007-AC-8` and `NFR-008-AC-8` restate that rule for their own
  axes.

Unbacked and unexecutable are disjoint and are never summed: an unbacked
criterion has nothing to run, an unexecutable one has something to run and no
way to run it today. A row whose Coverage Status reads test-backed is a statement
about the row's cited test cases, not a per-criterion guarantee; the `N of M`
figure in each Coverage Status cell is the per-criterion count and is the figure
to read.

### Why criteria stand unexecuted

Task-009 has a separately invoked producer-fixture smoke lane for its first
consumer increment. Its tests remain `#[ignore]` because they require A's
stripped release producer output and retain A's draft wire labels; they do not
change the planned status of FR-001, FR-002, or IT-001's full scenario matrix.

Every protocol, temporal and observation noun this repository's requirements use
resolves to an `ix://agent-ix/quire-specification/FR-2NN` reference rather than a
local definition. Where an upstream artifact records an open gap, the dependent
requirement cites the gap rather than supplying the missing identity, key or
order. Those citations are why several criteria below stand as obligations that
cannot yet be discharged — see `quire-specification` issues 28 and 29.

IT-001 is split into two rows because `/1` and `/2` are separate contracts that
may not substitute for each other. The `/2` row is planned and unexecuted. A's
`protocol_artifact::v2` is implemented and under final integration; its public
surface is recorded in IT-001 Preconditions and B builds against those exact
names, but the surface is not yet on `quire-spec-language` `origin/main` or any
remote branch, so B has not verified it directly and pins no intermediate commit.
SC-06, SC-06a, SC-07, SC-08 and SC-09 execute when A publishes its one final
immutable revision. The B-owned tests that exist for `/2`
(`tests/it001_v2_intake.rs`) are tagged `Task-009` only and carry no `IT-001`
trace tag: they assert B's own evidence loader and scenario corpus and make no
claim about A's behaviour, so they must not be counted as backing symbols for any
IT-001 scenario.

`FR-009`'s entry-status table records that every version-lock entry is
`unresolved` today. IT-002 and IT-003 declare FR-009 preconditions, so every one
of their scenarios is unexecutable. IT-001 declares no manifest precondition, so
its `/1` lane SC-01..SC-05 is planned rather than unexecutable, while its `/2`
lane is unexecutable for want of a publishable A revision.

## Functional Requirement Coverage

Parent and atomic rows are both listed. FR-010..FR-018 carry `object:` and are
in Domain Object Coverage instead.

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- |
| FR-001 | FR-001-AC-1, FR-001-AC-2, FR-001-AC-3, FR-001-AC-4, FR-001-AC-5 | TC-001 (AC-1..AC-4); IT-001 (AC-4, AC-5) | 🚧 Planned — 5 of 5 criteria backed, 0 unbacked. FR-001-AC-5 is integration-backed only and unexecutable: its sole backing is IT-001 SC-06/SC-06a, gated on A's final `/2` revision |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-5 | TC-002 (AC-1..AC-4); IT-001 (AC-5) | 🚧 Planned — 5 of 5 criteria backed, 0 unbacked. FR-002-AC-5 is integration-backed only and unexecutable: its sole backing is IT-001 SC-08, gated on A's final `/2` revision |
| FR-003 | FR-003-AC-1, FR-003-AC-2, FR-003-AC-3, FR-003-AC-4, FR-003-AC-5, FR-003-AC-6, FR-003-AC-7 | TC-003 (AC-1..AC-7); TC-009 (AC-7) | 🚧 Planned — 7 of 7 criteria test-backed, 0 unbacked, 0 unexecutable. Aggregate view of FR-019..FR-025 |
| FR-004 | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | TC-004 (AC-1..AC-4) | 🚧 Planned — 4 of 4 criteria test-backed, 0 unbacked, 0 unexecutable (FR-004-AC-3 landed by `tests/tc_004_closure_axes.rs`). Aggregate view of FR-026..FR-028 and FR-041 |
| FR-005 | FR-005-AC-1, FR-005-AC-2, FR-005-AC-3, FR-005-AC-4, FR-005-AC-5 | TC-005 (AC-1..AC-5) | 🚧 Planned — 5 of 5 criteria test-backed, 0 unbacked, 0 unexecutable. Aggregate view of FR-029 and FR-030 |
| FR-006 | FR-006-AC-1, FR-006-AC-2, FR-006-AC-3, FR-006-AC-4, FR-006-AC-5, FR-006-AC-6, FR-006-AC-7, FR-006-AC-8 | TC-006 (AC-1..AC-8) | 🚧 Planned — 8 of 8 criteria test-backed, 0 unbacked, 0 unexecutable. FR-006-AC-8 is the parent criterion added with the FR-031..FR-033 decomposition. Aggregate view of FR-031..FR-033 |
| FR-007 | FR-007-AC-1, FR-007-AC-2, FR-007-AC-3, FR-007-AC-4, FR-007-AC-5, FR-007-AC-6 | TC-007 (AC-1..AC-6) | 🚧 Planned — 6 of 6 criteria test-backed, 0 unbacked, 0 unexecutable. FR-007-AC-6 is the parent criterion added with the FR-034/FR-035 decomposition. Aggregate view of FR-034 and FR-035 |
| FR-008 | FR-008-AC-1, FR-008-AC-2, FR-008-AC-3, FR-008-AC-4, FR-008-AC-5, FR-008-AC-6, FR-008-AC-7, FR-008-AC-8, FR-008-AC-9 | TC-011 (AC-1..AC-8); none for FR-008-AC-9 | 🚧 Planned — 8 of 9 criteria test-backed. Unbacked: FR-008-AC-9, the parent criterion added with the FR-036..FR-040 decomposition, which verifies by Inspection and has no test case. Aggregate view of FR-036..FR-040 |
| FR-009 | FR-009-AC-1, FR-009-AC-2, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5, FR-009-AC-6, FR-009-AC-7 | TC-013 (AC-1..AC-7); TC-010 (AC-7); IT-002, IT-003 (AC-1..AC-6) | 🚧 Planned — 7 of 7 criteria test-backed, 0 unbacked, 0 unexecutable. TC-013 exercises resolution on B-owned fixtures with no external dependency, so no criterion depends on the IT rows alone; the IT-002/IT-003 lanes stay unexecutable on producer acceptance |
| FR-019 | FR-019-AC-1..AC-5 | TC-003 (AC-1..AC-5) | 🚧 Planned — 5 of 5 criteria test-backed, 0 unbacked, 0 unexecutable. FR-019-AC-1 also depends on the activation-axis contradiction `FR-231` routes to an owner ruling |
| FR-020 | FR-020-AC-1..AC-5 | TC-003 (AC-1..AC-5) | 🚧 Planned — 5 of 5 criteria test-backed, 0 unbacked, 0 unexecutable |
| FR-021 | FR-021-AC-1..AC-4 | TC-003 (AC-1..AC-4) | 🚧 Planned — 4 of 4 criteria test-backed, 0 unbacked, 0 unexecutable. Blocked on `FR-237`'s commit-boundary identity gap for promotion, not for backing |
| FR-022 | FR-022-AC-1..AC-5 | TC-003 (AC-1..AC-5) | 🚧 Planned — 5 of 5 criteria test-backed, 0 unbacked, 0 unexecutable |
| FR-023 | FR-023-AC-1..AC-5 | TC-003 (AC-1..AC-5) | 🚧 Planned — 5 of 5 criteria test-backed, 0 unbacked, 0 unexecutable |
| FR-024 | FR-024-AC-1..AC-5 | TC-003 (AC-1..AC-5) | 🚧 Planned — 5 of 5 criteria test-backed, 0 unbacked, 0 unexecutable |
| FR-025 | FR-025-AC-1..AC-3 | TC-003, TC-009 (AC-1..AC-3) | 🚧 Planned — 3 of 3 criteria test-backed, 0 unbacked, 0 unexecutable |
| FR-026 | FR-026-AC-1..AC-3 | TC-004 (AC-1, AC-2); none for FR-026-AC-3 | 🚧 Planned — 2 of 3 criteria test-backed. Unbacked: FR-026-AC-3 (Inspection) |
| FR-027 | FR-027-AC-1..AC-5 | TC-004 (AC-1..AC-4); none for FR-027-AC-5 | 🚧 Planned — 4 of 5 criteria test-backed. Unbacked: FR-027-AC-5 (Analysis) |
| FR-028 | FR-028-AC-1..AC-6 | TC-004 (AC-1..AC-6) | 🚧 Planned — 6 of 6 criteria test-backed, 0 unbacked, 0 unexecutable |
| FR-029 | FR-029-AC-1..AC-6 | TC-005 (AC-1..AC-5); none for FR-029-AC-6 | 🚧 Planned — 5 of 6 criteria test-backed. Unbacked: FR-029-AC-6 (Analysis) |
| FR-030 | FR-030-AC-1..AC-6 | TC-005 (AC-1..AC-5); none for FR-030-AC-6 | 🚧 Planned — 5 of 6 criteria test-backed. Unbacked: FR-030-AC-6 (Analysis) |
| FR-031 | FR-031-AC-1..AC-6 | TC-006 (AC-1, AC-2, AC-3, AC-5); none for FR-031-AC-4, FR-031-AC-6 | 🚧 Planned — 4 of 6 criteria test-backed. Unbacked: FR-031-AC-4, FR-031-AC-6 (Inspection) |
| FR-032 | FR-032-AC-1..AC-7 | TC-006 (AC-1..AC-7) | 🚧 Planned — 7 of 7 criteria test-backed, 0 unbacked, 0 unexecutable |
| FR-033 | FR-033-AC-1..AC-5 | TC-006 (AC-1..AC-5) | 🚧 Planned — 5 of 5 criteria test-backed, 0 unbacked, 0 unexecutable |
| FR-034 | FR-034-AC-1..AC-7 | TC-007 (AC-1..AC-5, AC-7); none for FR-034-AC-6 | 🚧 Planned — 6 of 7 criteria test-backed. Unbacked: FR-034-AC-6 (Inspection) |
| FR-035 | FR-035-AC-1..AC-7 | TC-007 (AC-1..AC-7) | 🚧 Planned — 7 of 7 criteria test-backed, 0 unbacked, 0 unexecutable |
| FR-036 | FR-036-AC-1..AC-6 | TC-011 (AC-1..AC-5); none for FR-036-AC-6 | 🚧 Planned — 5 of 6 criteria test-backed. Unbacked: FR-036-AC-6 (Inspection) |
| FR-037 | FR-037-AC-1..AC-5 | TC-011 (AC-1..AC-5) | 🚧 Planned — 5 of 5 criteria test-backed, 0 unbacked, 0 unexecutable |
| FR-038 | FR-038-AC-1..AC-6 | TC-011 (AC-1..AC-4); none for FR-038-AC-5, FR-038-AC-6 | 🚧 Planned — 4 of 6 criteria test-backed. Unbacked: FR-038-AC-5, FR-038-AC-6 (Inspection) |
| FR-039 | FR-039-AC-1..AC-6 | TC-011 (AC-1..AC-5); IT-003 (AC-5); none for FR-039-AC-6 | 🚧 Planned — 5 of 6 criteria test-backed. Unbacked: FR-039-AC-6 (Inspection). No criterion rests on IT-003 alone |
| FR-040 | FR-040-AC-1..AC-4 | TC-011 (AC-1..AC-4); IT-003 (AC-3) | 🚧 Planned — 4 of 4 criteria test-backed, 0 unbacked, 0 unexecutable |
| FR-041 | FR-041-AC-1..AC-4 | TC-004 (AC-1..AC-3); none for FR-041-AC-4 | 🚧 Planned — 3 of 4 criteria test-backed. Unbacked: FR-041-AC-4 (Analysis) |

Behavioural totals across these 32 rows: 177 acceptance criteria, of which 162
are backed by a test case, 2 are backed only by an integration scenario and both
of those are unexecutable (FR-001-AC-5 and FR-002-AC-5, each resting on IT-001
`/2` alone), and 13 are unbacked — 9 by Inspection and 4 by Analysis.
162 + 2 + 13 = 177.

## Domain Object Coverage

Requirements carrying `object:` specify the consumer-boundary vocabulary. They
are separated from the behavioural table because they define nouns rather than
behaviour.

Sixty-four criteria across the nine. Twenty-one of them name no test case and no
integration scenario — sixteen verify by Inspection and five by Analysis — and
this table writes `none` for each rather than a test-case id. The remaining
forty-three name a test case or an integration scenario, and each named test
case declares in its own Traceability section the object criteria it backs, so
the reference resolves in both directions.

| Object Req | Object type | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- | --- |
| FR-010 | domain | FR-010-AC-1..AC-3 | none — all three criteria are Inspection | 🚧 Planned — 0 of 3 criteria test-backed; unbacked: FR-010-AC-1, FR-010-AC-2, FR-010-AC-3 |
| FR-011 | value_object | FR-011-AC-1..AC-7 | TC-014 (AC-2..AC-6); none for FR-011-AC-1, FR-011-AC-7 | 🚧 Planned — 5 of 7 criteria test-backed; unbacked: FR-011-AC-1, FR-011-AC-7 (Inspection) |
| FR-012 | entity | FR-012-AC-1..AC-8 | TC-013 (AC-1..AC-6, AC-8); none for FR-012-AC-7 | 🚧 Planned — 7 of 8 criteria test-backed; unbacked: FR-012-AC-7 (Analysis) |
| FR-013 | value_object | FR-013-AC-1..AC-8 | TC-001 (AC-1, AC-2, AC-3, AC-5, AC-6); IT-001 (AC-1..AC-4, AC-6); none for FR-013-AC-7, FR-013-AC-8 | 🚧 Planned — 5 of 8 criteria test-backed; unbacked: FR-013-AC-7 (Inspection), FR-013-AC-8 (Analysis); unexecutable: FR-013-AC-4, whose sole backing is IT-001 SC-06 on A's `/2` |
| FR-014 | value_object | FR-014-AC-1..AC-8 | TC-011 (AC-1, AC-2, AC-4, AC-5, AC-7); IT-003 (AC-5, AC-6); none for FR-014-AC-3, FR-014-AC-8 | 🚧 Planned — 5 of 8 criteria test-backed; unbacked: FR-014-AC-3 (Analysis), FR-014-AC-8 (Inspection); unexecutable: FR-014-AC-6, whose sole backing is IT-003, blocked on Engineering Assurance #34 |
| FR-015 | value_object | FR-015-AC-1..AC-5 | TC-014 (AC-2, AC-3, AC-4); TC-001 (AC-2); none for FR-015-AC-1, FR-015-AC-5 | 🚧 Planned — 3 of 5 criteria test-backed; unbacked: FR-015-AC-1, FR-015-AC-5 (Inspection). The production and `/2` digest enums are two different vocabularies sharing one member name; FR-015 records the divergence as unreconciled |
| FR-016 | enumeration | FR-016-AC-1..AC-8 | TC-009 (AC-2..AC-7); TC-011 (AC-5); TC-008 (AC-6); TC-013 (AC-8); IT-001 (AC-2); none for FR-016-AC-1 | 🚧 Planned — 7 of 8 criteria test-backed; unbacked: FR-016-AC-1 (Inspection); 0 unexecutable |
| FR-017 | entity | FR-017-AC-1..AC-8 | TC-013 (AC-2); TC-008, TC-009 (AC-5); TC-010 (AC-7); TC-014 (AC-8); IT-002, IT-003 (AC-2); none for FR-017-AC-1, FR-017-AC-3, FR-017-AC-4, FR-017-AC-6 | 🚧 Planned — 4 of 8 criteria test-backed; unbacked: FR-017-AC-1, FR-017-AC-3, FR-017-AC-4 (Inspection), FR-017-AC-6 (Analysis); 0 unexecutable |
| FR-018 | entity | FR-018-AC-1..AC-9 | TC-013 (AC-1); IT-001 (AC-3, AC-5, AC-6, AC-8); none for FR-018-AC-2, FR-018-AC-4, FR-018-AC-7, FR-018-AC-9 | 🚧 Planned — 1 of 9 criteria test-backed; unbacked: FR-018-AC-2, FR-018-AC-4, FR-018-AC-9 (Inspection), FR-018-AC-7 (Analysis); unexecutable: FR-018-AC-3, FR-018-AC-5, FR-018-AC-6, FR-018-AC-8, which rest on IT-001 alone and index a pinned producer delivery that does not exist while every FR-009 entry is `unresolved` and A has published no `/2` revision |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
| --- | --- | --- | --- |
| NFR-001 | Property and metamorphic testing | TC-008 (AC-1..AC-3) | 🚧 Planned — 3 of 3 criteria test-backed, 0 unbacked, 0 unexecutable |
| NFR-002 | Property, mutation, and source-bound inventory inspection | TC-009 (AC-1..AC-4) | 🚧 Planned — 4 of 4 criteria test-backed, 0 unbacked, 0 unexecutable. IT-001 SC-07/SC-09 additionally verify this requirement and are unexecutable on A's final `/2` revision; no criterion rests on them alone |
| NFR-003 | Shared Rust language-boundary check, static analysis, architecture inspection, Rust review, Quire coverage, gap analysis | TC-010 (AC-1..AC-6); IT-003 (AC-4, AC-6) | 🚧 Planned — 6 of 6 criteria test-backed, 0 unbacked, 0 unexecutable |
| NFR-004 | Cargo metadata inspection, conditional-compilation inventory, enumerated gate execution over one unchanged head, per-criterion negative controls | TC-012 (AC-1..AC-10) | 🚧 Planned — 10 of 10 criteria backed by TC-012, 0 unbacked, 0 unexecutable. NFR-004-AC-2's cell reads `Inspection (TC-012)`, so it is inspection-method but test-case-backed. IT-003 previously appeared here but declares and exercises no NFR-004 scenario |
| NFR-005 | Static inspection and Quire coverage over the integration lanes, denominated on owed rather than executed scenarios | IT-002, IT-003 | 🚧 Planned — this requirement states five metrics and no acceptance criteria, so it contributes 0 to the 288. Its denominator is the 22 scenarios IT-002 and IT-003 owe (12 + 10); all 22 are unexecutable, so every metric scores 0 of 22 rather than a vacuous 100% |
| NFR-006 | Cargo manifest, lockfile and rights-record inspection with per-criterion negative controls; delivery-surface tests on B-owned fixtures | TC-013 (AC-6, AC-7, AC-8); none for NFR-006-AC-1..AC-5 | 🚧 Planned — 3 of 8 criteria test-backed; unbacked: NFR-006-AC-1, NFR-006-AC-2, NFR-006-AC-3, NFR-006-AC-4, NFR-006-AC-5 (Inspection); 0 unexecutable, because TC-013's delivery-surface controls run on B-owned fixtures and need no producer delivery. NFR-006-AC-4 is recorded by its own text as failing on the tree as it stands, a `DEPENDENCIES.md`/`Cargo.toml` revision divergence; this matrix claims no execution either way |
| NFR-007 | Public-surface inspection, two-revision surface analysis, and IT-001 cross-version and inherited-axis scenarios | IT-001 (AC-3, AC-4, AC-5, AC-7); none for NFR-007-AC-1, NFR-007-AC-2, NFR-007-AC-6, NFR-007-AC-8 | 🚧 Planned — 0 of 8 criteria test-backed; 4 integration-backed and all 4 unexecutable: NFR-007-AC-3, NFR-007-AC-4 and NFR-007-AC-7 rest on IT-001 SC-06 and NFR-007-AC-5 on IT-001 SC-06a, and A has published no `quire.compiled-protocol/2` revision B can pin. Unbacked: NFR-007-AC-1, NFR-007-AC-2, NFR-007-AC-8 (Inspection), NFR-007-AC-6 (Analysis). NFR-007-AC-8 is the criterion that requires exactly this reporting |
| NFR-008 | Static site counts over the repository's own sources, a rendering-replacement mutation, a captured-stream control, an accessor compilation control, and reproduction comparison | TC-011 (AC-1, AC-2, AC-6); TC-008 (AC-7); none for NFR-008-AC-3, NFR-008-AC-4, NFR-008-AC-5, NFR-008-AC-8 | 🚧 Planned — 4 of 8 criteria test-backed; unbacked: NFR-008-AC-3, NFR-008-AC-4, NFR-008-AC-5, NFR-008-AC-8 (Inspection); 0 unexecutable |

Non-functional totals across these 8 rows: 47 acceptance criteria, of which 30
are backed by a test case, 4 are backed only by an integration scenario and all
4 of those are unexecutable, and 13 are unbacked — 12 by Inspection and 1 by
Analysis. 30 + 4 + 13 = 47. NFR-005 adds no criterion to that count, because it
states five metrics and no acceptance criteria, and is denominated on owed
scenarios instead.

Across all three coverage tables the three per-criterion figures reconcile to
the owed total: 162 + 37 + 30 = 229 test-backed, 2 + 6 + 4 = 12
integration-backed and unexecutable, and 13 + 21 + 13 = 47 unbacked, summing to
the 288 authored acceptance criteria.

## Integration Coverage

| Integration Test | Boundary | Requirements | Status |
| --- | --- | --- | --- |
| IT-001 `/1` | Native compiler `quire.compiled-protocol/1` artifact to protocol library | FR-001, FR-002, FR-013, FR-016, NFR-002 | 🚧 Planned — SC-01..SC-05, 5 owed scenarios; fixture lane `#[ignore]`. IT-001 declares no FR-009 manifest precondition, so this lane is planned rather than unexecutable |
| IT-001 `/2` | Native compiler `/2` artifact to protocol library | FR-001-AC-5, FR-002-AC-5, FR-013-AC-4, FR-018-AC-3, FR-018-AC-5, FR-018-AC-6, FR-018-AC-8, NFR-002, NFR-007 | 🚧 Planned — SC-06, SC-06a, SC-07, SC-08, SC-09, 5 owed scenarios, all 5 unexecutable and scoring 0 of 5. A's `protocol_artifact::v2` is implemented and under final integration, not yet published to a remote, so no revision can be pinned. SC-06 and SC-06a verify NFR-007; SC-06a re-runs SC-01..SC-05 as its regression baseline |
| IT-002 | Observation and temporal facts to protocol evaluator/result | FR-003, FR-004, FR-006, FR-009, FR-017, NFR-002, NFR-005 | 🚧 Planned — SC-01, SC-02, SC-03, SC-04, SC-05a..SC-05d, SC-06, SC-07a, SC-07b, SC-08, 12 owed scenarios, all 12 unexecutable and scoring 0 of 12; blocked on accepted F/E producer contracts and on every FR-009 entry being `unresolved` |
| IT-003 | Protocol result through Engineering Assurance, E02, and Quoin | FR-006, FR-007, FR-009, FR-014, FR-017, FR-039, FR-040, NFR-002, NFR-003, NFR-005 | 🚧 Planned — SC-01..SC-07, SC-08a, SC-08b, SC-09, 10 owed scenarios, all 10 unexecutable and scoring 0 of 10; blocked on Engineering Assurance #34 |

Owed integration scenarios: 32 across the three integration tests (IT-001 10,
IT-002 12, IT-003 10). 5 are planned and 27 are unexecutable. No scenario is
reported as passing or failing.

## Test Case Summary

`Traces To` names every requirement whose acceptance criteria cite the test case,
parents and atomic parts alike. A test case whose Traceability section does not
yet name every requirement listed here is a back-edge gap in that test case, not
a coverage claim in this matrix.

| Test ID | Title | Type | Priority | Traces To | Status |
| --- | --- | --- | --- | --- | --- |
| TC-001 | Admit only the selected protocol profile | Property | P0 | FR-001, FR-013, FR-015 | 🚧 Planned |
| TC-002 | Link and discriminate finite protocol structure | Property | P0 | FR-002 | 🚧 Planned (FR-002-AC-1 identity set executing; AC-2..AC-4 unwritten) |
| TC-003 | Assess compensation, retry, commit, and recovery | Property | P0 | FR-003, FR-019, FR-020, FR-021, FR-022, FR-023, FR-024, FR-025 | 🚧 Planned — backs the parent and all seven atomic parts; its Traceability section names only FR-003 today |
| TC-004 | Assess finite global conformance | Property | P0 | FR-004, FR-026, FR-027, FR-028, FR-041 | 🚧 Planned (FR-004-AC-3 landed; AC-1/AC-2/AC-4 pending the batch and incremental evaluators) |
| TC-005 | Separate protocol claim dispositions | Property | P0 | FR-005, FR-029, FR-030 | 🚧 Planned |
| TC-006 | Produce and strictly read canonical protocol results | Property | P0 | FR-006, FR-031, FR-032, FR-033 | 🚧 Planned |
| TC-007 | Project canonical protocol results to E02 | Property | P0 | FR-007, FR-034, FR-035 | 🚧 Planned |
| TC-008 | Reproduce results under equivalent presentations | Property | P0 | NFR-001, FR-016, FR-017, NFR-008 | 🚧 Planned — reproduction-record slice executed (12 tests, chunk C-31); equivalent-presentation permutations and the full one-axis mutation matrix remain planned |
| TC-009 | Enforce every protocol resource boundary | Property | P0 | NFR-002, FR-003, FR-016, FR-017, FR-025 | 🚧 Planned |
| TC-010 | Preserve Rust and shared assurance containment | Static | P0 | NFR-003, FR-009, FR-017 | 🚧 Planned |
| TC-011 | Return typed refusals without text parsing | Property | P0 | FR-008, FR-014, FR-016, FR-036, FR-037, FR-038, FR-039, FR-040, NFR-008 | 🚧 Planned — backs FR-008-AC-1..AC-8 and the five atomic parts FR-036..FR-040; FR-008-AC-9 is unbacked |
| TC-012 | Run the local verification gate and refuse an undefined lane | Static | P0 | NFR-004 | 🚧 Planned |
| TC-013 | Resolve the version-lock manifest | Property | P0 | FR-009, FR-012, FR-016, FR-017, FR-018, NFR-006 | 🚧 Planned — B-owned fixtures only; runnable without any external producer delivery |
| TC-014 | Apply the consumer identity discipline | Property | P0 | FR-011, FR-015, FR-017 | 🚧 Planned — B-owned fixtures only; runnable without any external producer delivery |

## Test Execution Summary

Row counts, recomputed from the tree. Every matrix row above is counted exactly
once, and `Blocked` means unexecutable — neither passing nor failing.

| Category | Total | Passed | Failed | Blocked | Planned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Functional requirement rows | 32 | 0 | 0 | 0 | 32 |
| Domain object requirement rows | 9 | 0 | 0 | 0 | 9 |
| Non-functional requirement rows | 8 | 0 | 0 | 0 | 8 |
| Integration rows | 4 | 0 | 0 | 3 | 1 |
| Test case rows | 14 | 0 | 0 | 0 | 14 |
| Overall | 67 | 0 | 0 | 3 | 64 |

The Integration count is four because IT-001 carries a separate `/1` and `/2`
row; the two contracts may not substitute for each other, so one row cannot
report both. Three of the four are unexecutable: IT-001 `/2`, IT-002 and IT-003.

### Owed acceptance criteria

`NFR-005` denominates on what each lane **owes**, not on what it executed, so a
fully blocked integration cannot score 100%. These totals follow that rule: the
denominator is every authored acceptance criterion, and an unexecutable
criterion is a zero inside it rather than a row removed from it.

| Category | Total | Passed | Failed | Blocked | Planned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Criteria backed by a test case | 229 | 0 | 0 | 0 | 229 |
| Criteria backed only by an integration scenario | 12 | 0 | 0 | 12 | 0 |
| Criteria with no test case and no integration scenario | 47 | 0 | 0 | 0 | 47 |
| Overall owed criteria | 288 | 0 | 0 | 12 | 276 |

The 12 unexecutable criteria are FR-001-AC-5, FR-002-AC-5, FR-013-AC-4,
FR-018-AC-3, FR-018-AC-5, FR-018-AC-6, FR-018-AC-8, NFR-007-AC-3, NFR-007-AC-4,
NFR-007-AC-5 and NFR-007-AC-7 — eleven waiting on a
`quire.compiled-protocol/2` revision B can pin — and FR-014-AC-6, waiting on
IT-003 and Engineering Assurance #34.

The 47 unbacked criteria are FR-008-AC-9, FR-010-AC-1, FR-010-AC-2,
FR-010-AC-3, FR-011-AC-1, FR-011-AC-7, FR-012-AC-7, FR-013-AC-7, FR-013-AC-8,
FR-014-AC-3, FR-014-AC-8, FR-015-AC-1, FR-015-AC-5, FR-016-AC-1, FR-017-AC-1,
FR-017-AC-3, FR-017-AC-4, FR-017-AC-6, FR-018-AC-2, FR-018-AC-4, FR-018-AC-7,
FR-018-AC-9, FR-026-AC-3, FR-027-AC-5, FR-029-AC-6, FR-030-AC-6, FR-031-AC-4,
FR-031-AC-6, FR-034-AC-6, FR-036-AC-6, FR-038-AC-5, FR-038-AC-6, FR-039-AC-6,
FR-041-AC-4, NFR-006-AC-1, NFR-006-AC-2, NFR-006-AC-3, NFR-006-AC-4,
NFR-006-AC-5, NFR-007-AC-1, NFR-007-AC-2, NFR-007-AC-6, NFR-007-AC-8,
NFR-008-AC-3, NFR-008-AC-4, NFR-008-AC-5 and NFR-008-AC-8. Each verifies by
Inspection or Analysis and none is claimed as test coverage. No row cites a
test-case id above TC-014, and none is invented to close one of these gaps.
