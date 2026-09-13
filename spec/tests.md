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

The five words below are not interchangeable, and no total mixes them.

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
- **Engine-bound** — a compiler-checked trace symbol actually resolves to the
  criterion. This is the only one of the five words that is *measured* rather
  than read off a `Verification` cell, and it is much smaller than the others:

  ```
  $ quire coverage --scope /home/peter/dev/quire-protocol
  Coverage: 37/322 rows backed (11%)
  ```

  Thirty-seven rows: FR-001 3 of 5, FR-002 3 of 5, FR-004 1 of 4, FR-005 1 of 5,
  FR-006 1 of 8, FR-008 6 of 9, FR-011 3 of 7, FR-015 3 of 5, FR-028 5 of 6,
  NFR-001 2 of 3, NFR-002 2 of 4, plus 7 of the 14 test-case rows.

  FR-002 was 0 of 5 at the previous measurement and is 3 of 5 now. It had been 1
  of 5 before that: the row that went was `FR-002-AC-1`, which
  `tests/it001_native_handoff.rs` tagged, and the owner ruled that criterion
  unresolved until runtime instance identities exist (issue #4), so the tag went
  and the figure fell by one. It fell because the measurement got more honest,
  not because anything stopped working — that test is `#[ignore]`d, so the
  criterion was never checked by a run in the first place. The three now bound
  are C-12's, from `src/link/` and `tests/tc_002_identities.rs`; `FR-002-AC-1`
  is still not among them and stays unbound until #4 resolves.

  The gap between 229 test-backed and 37 engine-bound is not a contradiction —
  the two words mean different things, and the definitions above say so — but it
  is the number that says how much of this specification a test currently
  reaches. **Read it as the coverage figure.** "Test-backed" says a criterion has
  somewhere to be tested; "engine-bound" says something tests it.

  The gap closes by tagging, never by editing this table. A trace attribute is
  the only thing that binds a test to a criterion, so a criterion a test
  exercises without naming it stays unbound however this matrix describes it.
  Tagging `tc_004`, `tc_006`, `tc_002` and `tc_008` at criterion level took
  coverage from 13 rows to 24 without changing one line of test logic, and
  removing one criterion a test could not check took it back to 23.

  Two kinds of unbound symbol remain, and they are not the same thing:

  - **Deliberately unbound.** The 24 tests in `tests/it001_v2_intake.rs` carry
    `Task-009` and no criterion, because they exercise this repository's own
    fixture loader rather than any IT-001 scenario. Tagging them `IT-001` would
    make 24 unexecuted scenarios look covered. They stay as they are. The 13 tests
    in `tests/it001_v2_handoff.rs` carry `Task-012` for the same reason. A's
    revision is promoted now, so the remaining reason is retention: the 6 that
    submit to A are `#[ignore]`d because the handoff is configured out of band,
    and the 7 that exercise the integrity mechanism build their own synthetic
    handoffs under `CARGO_TARGET_TMPDIR` and run on every gate.
  - **Still under-tagged.** Tests that exercise a criterion they do not name.
    `tests/it001_native_handoff.rs` is the largest remaining case, at 24 traces
    over 4 criteria.

- **Unexecutable** — a backing exists but cannot be run, so the criterion is
  neither passing nor failing. 12 of 288 criteria, all of them
  integration-backed. Under `FR-009-AC-3` such a scenario is reported
  unexecutable and counted as zero, never as passing and never omitted from a
  denominator. `NFR-007-AC-8` and `NFR-008-AC-8` restate that rule for their own
  axes.

Unbacked and unexecutable are disjoint and are never summed: an unbacked
criterion has nothing to run, an unexecutable one has something to run and no
way to run it today. A row whose Status reads test-backed is a statement
about the row's cited test cases, not a per-criterion guarantee; the `N of M`
figure in each Status cell is the per-criterion count and is the figure
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

IT-001 is split into two rows because version 1 and version 2 are separate contracts that
may not substitute for each other. The version 2 row is planned and unexecuted.

A has now published a `/2` handoff, and B has run it: `tests/it001_v2_handoff.rs`
admits and links A's offered bytes against an expectation built only from A's
independent selection sidecar and the original inputs, and submits all 28 cases
of A's mutation manifest. On the revision named below, the base offer is admitted
and every mutation refuses with the identity A declares for it.

Of the two conditions that held this back, one is now met and one is not, so the
outcome remains **candidate evidence, not a discharged scenario**, and neither
the version 2 row nor any criterion resting on it is scored against it:

- **Met.** The pin is now `quire-spec-language` `04254a3`, that repository's
  `main` after PR #75 merged, and the handoff is committed there under
  `artifacts/compiled-protocol-v2/`. This is A's promoted revision, and its
  published `SHA256SUMS` covers all 109 members; the lane verifies every one of
  them, and refuses any member the manifest does not list, before reading any.
- **Not met.** The handoff directory is not retained in this repository. The lane
  reads it from `QUIRE_PROTOCOL_A_V2_HANDOFF`, and the 6 tests that submit to A
  are `#[ignore]`d on it, so with the variable unset the local gate sees
  `7 passed; 6 ignored` — six scenarios reported unexecuted and counted as zero,
  as `FR-009-AC-3` requires, never as passing. A scenario that cannot execute
  under the gate on a clean checkout is not executed coverage. Vendoring A's 13 MB
  of published artifacts here would mirror what its owner already publishes, so
  the resolution asked of A is a published path constant naming the handoff
  inside its own crate source, which every dependent already has on disk. Filed
  as `agent-ix/quire-spec-language#78`.

The version 2 lane's tests are therefore tagged `Task-012` only and carry no
`IT-001` trace tag, for the same reason the loader tests in
`tests/it001_v2_intake.rs` are tagged `Task-009` only: they must not be counted
as backing symbols for any IT-001 scenario until both conditions above are met.

`FR-009`'s entry-status table records that every version-lock entry is
`unresolved` today. IT-002 and IT-003 declare FR-009 preconditions, so every one
of their scenarios is unexecutable. IT-001 declares no manifest precondition, so
its version 1 lane SC-01..SC-05 is planned rather than unexecutable, while its version 2
lane is unexecutable for want of a handoff this repository can reach under the
declared gate; A's immutable revision is promoted and pinned at `04254a3`.

## Functional Requirement Coverage

Parent and atomic rows are both listed. FR-010..FR-018 carry `object:` and are
in Domain Object Coverage instead.

| Functional Req | Acceptance Criteria | Test Cases | Status |
| --- | --- | --- | --- |
| FR-001 | FR-001-AC-1, FR-001-AC-2, FR-001-AC-3, FR-001-AC-4, FR-001-AC-5 | TC-001 (AC-1..AC-4); IT-001 (AC-4, AC-5) | 🚧 Planned — 5 of 5 criteria backed, 0 unbacked. FR-001-AC-5 is integration-backed only and unexecutable: its sole backing is IT-001 SC-06/SC-06a, gated not on A's revision, which is pinned at `04254a3`, but on a handoff this repository can reach under the declared gate |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-5 | TC-002 (AC-1..AC-4); IT-001 (AC-5) | 🚧 Planned — 5 of 5 criteria backed, 0 unbacked. FR-002-AC-5 is integration-backed only and unexecutable: its sole backing is IT-001 SC-08, gated not on A's revision, which is pinned at `04254a3`, but on a handoff this repository can reach under the declared gate. The identity mapping landed as chunk C-12: AC-4 executes in both directions and AC-3 executes partially. AC-2 landed as chunk C-14 and is now test-backed: `src/link/interleaving.rs` enumerates the linear extensions of the causal order A retained, excluding the `repeat_progress` back edge A documents as its explicit cyclic edge, and TC-002 asserts every one preserves the authored edges and satisfies the declared join. C-14 also closed part of an AC-4 defect: five channel premise axes — sender, receiver, message type, ordering policy and delivery cardinality — were not identity-bearing, and `ChannelPremises` now carries them. AC-4 is therefore **partially backed, not closed**. FR-216 fixes the channel premise key at six components; A's `Channel` at the pin publishes five, and no field carries a duplication premise separate from the delivery interval. Deriving one would be this crate interpreting A's numerals, which FR-216 Absence forbids, so the contradiction is asked rather than absorbed: `agent-ix/quire-spec-language` issue 85 puts it to the producer, and FR-216's own Open gap 1 records the specification half. Until one of them answers, the sixth component has no authored field to mutate and no axis in the sweep. FR-002's refusal list divides three ways — A refuses the causal edge set, joins, cycles and reference resolution inside `read`; this crate refuses zero-progress iteration and duplicate role authorities, which A leaves open; and unreachable termination, overlapping choices and owner-unobservable choices are refused by nobody, because no reachability analysis exists in either repository and choice disjointness is a decision procedure over arbitrary guards that neither has. Those three are AC-3 obligations no code discharges. AC-1 is unresolved by owner ruling on issue #4: it requires distinct runtime instance identities, and A carries binding requirements rather than instances, so no test here can establish it |
| FR-003 | FR-003-AC-1, FR-003-AC-2, FR-003-AC-3, FR-003-AC-4, FR-003-AC-5, FR-003-AC-6, FR-003-AC-7 | TC-003 (AC-1..AC-7); TC-009 (AC-7) | 🚧 Planned — 7 of 7 criteria test-backed, 0 unbacked, 0 unexecutable. Aggregate view of FR-019..FR-025 |
| FR-004 | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | TC-004 (AC-1..AC-4) | 🚧 Planned — 4 of 4 criteria test-backed, 0 unbacked, 0 unexecutable (FR-004-AC-3 landed by `tests/tc_004_closure_axes.rs`). Aggregate view of FR-026..FR-028 and FR-041 |
| FR-005 | FR-005-AC-1, FR-005-AC-2, FR-005-AC-3, FR-005-AC-4, FR-005-AC-5 | TC-005 (AC-1..AC-5) | 🚧 Planned — 5 of 5 criteria test-backed, 0 unbacked, 0 unexecutable. FR-005-AC-1's disposition shape landed as chunk C-22 in `tests/tc_005_claim_dispositions.rs`; its end-to-end scenario and AC-2..AC-5 are pending. Aggregate view of FR-029 and FR-030 |
| FR-006 | FR-006-AC-1, FR-006-AC-2, FR-006-AC-3, FR-006-AC-4, FR-006-AC-5, FR-006-AC-6, FR-006-AC-7, FR-006-AC-8 | TC-006 (AC-1..AC-8) | 🚧 Planned — 8 of 8 criteria test-backed, 0 unbacked, 0 unexecutable. FR-006-AC-8 is the parent criterion added with the FR-031..FR-033 decomposition. Aggregate view of FR-031..FR-033 |
| FR-007 | FR-007-AC-1, FR-007-AC-2, FR-007-AC-3, FR-007-AC-4, FR-007-AC-5, FR-007-AC-6 | TC-007 (AC-1..AC-6) | 🚧 Planned — 6 of 6 criteria test-backed, 0 unbacked, 0 unexecutable. FR-007-AC-6 is the parent criterion added with the FR-034/FR-035 decomposition. Aggregate view of FR-034 and FR-035 |
| FR-008 | FR-008-AC-1, FR-008-AC-2, FR-008-AC-3, FR-008-AC-4, FR-008-AC-5, FR-008-AC-6, FR-008-AC-7, FR-008-AC-8, FR-008-AC-9 | TC-011 (AC-1..AC-8); none for FR-008-AC-9 | 🚧 Planned — 8 of 9 criteria test-backed. Unbacked: FR-008-AC-9, the parent criterion added with the FR-036..FR-040 decomposition, which verifies by Inspection and has no test case. FR-008-AC-1, AC-2 and AC-7 landed in `tests/tc_011_typed_refusals.rs`, as did the structural halves of AC-4 and AC-8. Aggregate view of FR-036..FR-040 |
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
version 2 alone), and 13 are unbacked — 9 by Inspection and 4 by Analysis.
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

| Object Req | Object type | Acceptance Criteria | Test Cases | Status |
| --- | --- | --- | --- | --- |
| FR-010 | domain | FR-010-AC-1..AC-3 | none — all three criteria are Inspection | 🚧 Planned — 0 of 3 criteria test-backed; unbacked: FR-010-AC-1, FR-010-AC-2, FR-010-AC-3 |
| FR-011 | value_object | FR-011-AC-1..AC-7 | TC-014 (AC-2..AC-6); none for FR-011-AC-1, FR-011-AC-7 | 🚧 Planned — 5 of 7 criteria test-backed; unbacked: FR-011-AC-1, FR-011-AC-7 (Inspection) |
| FR-012 | entity | FR-012-AC-1..AC-8 | TC-013 (AC-1..AC-6, AC-8); none for FR-012-AC-7 | 🚧 Planned — 7 of 8 criteria test-backed; unbacked: FR-012-AC-7 (Analysis) |
| FR-013 | value_object | FR-013-AC-1..AC-8 | TC-001 (AC-1, AC-2, AC-3, AC-5, AC-6); IT-001 (AC-1..AC-4, AC-6); none for FR-013-AC-7, FR-013-AC-8 | 🚧 Planned — 5 of 8 criteria test-backed; unbacked: FR-013-AC-7 (Inspection), FR-013-AC-8 (Analysis); unexecutable: FR-013-AC-4, whose sole backing is IT-001 SC-06 on the version 2 |
| FR-014 | value_object | FR-014-AC-1..AC-8 | TC-011 (AC-1, AC-2, AC-4, AC-5, AC-7); IT-003 (AC-5, AC-6); none for FR-014-AC-3, FR-014-AC-8 | 🚧 Planned — 5 of 8 criteria test-backed; unbacked: FR-014-AC-3 (Analysis), FR-014-AC-8 (Inspection); unexecutable: FR-014-AC-6, whose sole backing is IT-003, blocked on Engineering Assurance #34 |
| FR-015 | value_object | FR-015-AC-1..AC-5 | TC-014 (AC-2, AC-3, AC-4); TC-001 (AC-2); none for FR-015-AC-1, FR-015-AC-5 | 🚧 Planned — 3 of 5 criteria test-backed; unbacked: FR-015-AC-1, FR-015-AC-5 (Inspection). The production and version 2 digest enums are two different vocabularies sharing one member name; FR-015 records the divergence as unreconciled |
| FR-016 | enumeration | FR-016-AC-1..AC-8 | TC-009 (AC-2..AC-7); TC-011 (AC-5); TC-008 (AC-6); TC-013 (AC-8); IT-001 (AC-2); none for FR-016-AC-1 | 🚧 Planned — 7 of 8 criteria test-backed; unbacked: FR-016-AC-1 (Inspection); 0 unexecutable |
| FR-017 | entity | FR-017-AC-1..AC-8 | TC-013 (AC-2); TC-008, TC-009 (AC-5); TC-010 (AC-7); TC-014 (AC-8); IT-002, IT-003 (AC-2); none for FR-017-AC-1, FR-017-AC-3, FR-017-AC-4, FR-017-AC-6 | 🚧 Planned — 4 of 8 criteria test-backed; unbacked: FR-017-AC-1, FR-017-AC-3, FR-017-AC-4 (Inspection), FR-017-AC-6 (Analysis); 0 unexecutable |
| FR-018 | entity | FR-018-AC-1..AC-9 | TC-013 (AC-1); IT-001 (AC-3, AC-5, AC-6, AC-8); none for FR-018-AC-2, FR-018-AC-4, FR-018-AC-7, FR-018-AC-9 | 🚧 Planned — 1 of 9 criteria test-backed; unbacked: FR-018-AC-2, FR-018-AC-4, FR-018-AC-9 (Inspection), FR-018-AC-7 (Analysis); unexecutable: FR-018-AC-3, FR-018-AC-5, FR-018-AC-6, FR-018-AC-8, which rest on IT-001 alone and index a pinned producer delivery that does not exist while every FR-009 entry is `unresolved` and A's `/2` handoff, though now pinned to A's promoted `main` revision `04254a3`, is not retained in this repository and so does not execute under the gate |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
| --- | --- | --- | --- |
| NFR-001 | Property and metamorphic testing | TC-008 (AC-1..AC-3) | 🚧 Planned — 3 of 3 criteria test-backed, 0 unbacked, 0 unexecutable |
| NFR-002 | Property, mutation, and source-bound inventory inspection | TC-009 (AC-1..AC-4) | 🚧 Planned — 4 of 4 criteria test-backed, 0 unbacked, 0 unexecutable. IT-001 SC-07/SC-09 additionally verify this requirement and are unexecutable for want of a reachable handoff, not for want of A's revision; no criterion rests on them alone. The count, allocation and deadline guard slice executed as chunk C-29; the source-bound site inventory is chunk C-30 and remains planned |
| NFR-003 | Shared Rust language-boundary check, static analysis, architecture inspection, Rust review, Quire coverage, gap analysis | TC-010 (AC-1..AC-6); IT-003 (AC-4, AC-6) | 🚧 Planned — 6 of 6 criteria test-backed, 0 unbacked, 0 unexecutable |
| NFR-004 | Cargo metadata inspection, conditional-compilation inventory, enumerated gate execution over one unchanged head, per-criterion negative controls | TC-012 (AC-1..AC-10) | 🚧 Planned — 10 of 10 criteria backed by TC-012, 0 unbacked, 0 unexecutable. NFR-004-AC-2's cell reads `Inspection (TC-012)`, so it is inspection-method but test-case-backed. IT-003 previously appeared here but declares and exercises no NFR-004 scenario |
| NFR-005 | Static inspection and Quire coverage over the integration lanes, denominated on owed rather than executed scenarios | IT-002, IT-003 | 🚧 Planned — this requirement states five metrics and no acceptance criteria, so it contributes 0 to the 288. Its denominator is the 22 scenarios IT-002 and IT-003 owe (12 + 10); all 22 are unexecutable, so every metric scores 0 of 22 rather than a vacuous 100% |
| NFR-006 | Cargo manifest, lockfile and rights-record inspection with per-criterion negative controls; delivery-surface tests on B-owned fixtures | TC-013 (AC-6, AC-7, AC-8); none for NFR-006-AC-1..AC-5 | 🚧 Planned — 3 of 8 criteria test-backed; unbacked: NFR-006-AC-1, NFR-006-AC-2, NFR-006-AC-3, NFR-006-AC-4, NFR-006-AC-5 (Inspection); 0 unexecutable, because TC-013's delivery-surface controls run on B-owned fixtures and need no producer delivery. NFR-006-AC-4 is recorded by its own text as failing on the tree as it stands, a `DEPENDENCIES.md`/`Cargo.toml` revision divergence; this matrix claims no execution either way |
| NFR-007 | Public-surface inspection, two-revision surface analysis, and IT-001 cross-version and inherited-axis scenarios | IT-001 (AC-3, AC-4, AC-5, AC-7); none for NFR-007-AC-1, NFR-007-AC-2, NFR-007-AC-6, NFR-007-AC-8 | 🚧 Planned — 0 of 8 criteria test-backed; 4 integration-backed and all 4 unexecutable: NFR-007-AC-3, NFR-007-AC-4 and NFR-007-AC-7 rest on IT-001 SC-06 and NFR-007-AC-5 on IT-001 SC-06a, and while B now pins A's promoted revision `04254a3`, the handoff it names is not retained in this repository and so does not execute under the gate. Unbacked: NFR-007-AC-1, NFR-007-AC-2, NFR-007-AC-8 (Inspection), NFR-007-AC-6 (Analysis). NFR-007-AC-8 is the criterion that requires exactly this reporting |
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
| IT-001 version 1 | Native compiler `quire.compiled-protocol/1` artifact to protocol library | FR-001, FR-002, FR-013, FR-016, NFR-002 | 🚧 Planned — SC-01..SC-05, 5 owed scenarios; fixture lane `#[ignore]`. IT-001 declares no FR-009 manifest precondition, so this lane is planned rather than unexecutable |
| IT-001 version 2 | Native compiler version 2 artifact to protocol library | FR-001-AC-5, FR-002-AC-5, FR-013-AC-4, FR-018-AC-3, FR-018-AC-5, FR-018-AC-6, FR-018-AC-8, NFR-002, NFR-007 | 🚧 Planned — SC-06, SC-06a, SC-07, SC-08, SC-09, 5 owed scenarios, all 5 unexecutable under the gate and scoring 0 of 5. A's `/2` handoff has been run out of band against `quire-spec-language` `04254a3`, that repository's promoted `main`: its published `SHA256SUMS` verified over all 109 members, base admission succeeded, and all 28 declared mutations refused with their declared identities. That is candidate evidence only — the handoff is not retained in this repository, so it does not execute under the gate and no scenario is scored from it. SC-06 and SC-06a verify NFR-007; SC-06a re-runs SC-01..SC-05 as its regression baseline |
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
| TC-002 | Link and discriminate finite protocol structure | Property | P0 | FR-002 | 🚧 Planned — the linked mapping executes in 20 tests in `src/link/mod.rs` and 6 in `tests/tc_002_identities.rs`, tagged `Task-002`, `FR-002-AC-3` or `FR-002-AC-4`, and none claiming FR-002-AC-1. An earlier revision of this row said "all tagged `Task-002`" while fourteen carried a criterion tag, and said none claimed FR-002-AC-1 while `tc_002_two_packages_do_not_share_identities` did — so `quire coverage` counted FR-002 backed partly on a tag FR-002's own text rules out. The tag is now `Task-002`. The five added by C-13 cover accumulation: every zero-progress repeat reported, every failing channel premise reported, a gating refusal accumulating with nothing, every finding over artifact content carrying its region while the two refusals of the caller's `declaration` argument carry none, and accumulated findings still emitting no linked representation. C-14 added twelve more, in two files: `src/link/mod.rs` goes to 28 and the new `src/link/interleaving.rs` carries 4, for 32 in the unit binary. Nine tagged `FR-002-AC-2` prove the interleaving obligation — over a two-branch parallel fixture, all 924 = C(12,6) linear extensions of the retained causal order are admitted, every one puts each branch named by the parallel's own `join` field before the parallel's exit, dropping a join edge admits a branch completing late, and permuting A's edge table changes nothing; over hand-built graphs, a cycle admits none and the repeat back edge is not an ordering. Two of the nine guard the enumerator's domain: A derives `branch` and `join` edges identically for a `Parallel` and a `Choice`, so an edge set spanning a choice, repeat or await is refused rather than enumerated — ordering a choice's cases would report a run taking both. The sweep is three tests over one 30-axis table: `tc_002_every_refused_axis_is_refused_at_its_source` (7 axes, `FR-002-AC-3`, comparing each finding's `locus` against a fixture whose records each carry a distinct region), `tc_002_every_semantic_axis_changes_the_linked_protocol` (18 axes, `FR-002-AC-4`) and `tc_002_the_unguarded_axes_are_still_unguarded` (5 axes, `Task-002`). The last is deliberately tagged to no criterion: every assertion in it says a mutation is *accepted*, either because A refuses it first or because nobody does, and tagging non-refusal to a refusal criterion is how this row's own history records FR-002 being miscounted before |
| TC-003 | Assess compensation, retry, commit, and recovery | Property | P0 | FR-003, FR-019, FR-020, FR-021, FR-022, FR-023, FR-024, FR-025 | 🚧 Planned — backs the parent and all seven atomic parts; its Traceability section names only FR-003 today |
| TC-004 | Assess finite global conformance | Property | P0 | FR-004, FR-026, FR-027, FR-028, FR-041 | 🚧 Planned (FR-004-AC-3 landed; AC-1/AC-2/AC-4 pending the batch and incremental evaluators) |
| TC-005 | Separate protocol claim dispositions | Property | P0 | FR-005, FR-029, FR-030 | 🚧 Planned — the six-disposition record and its no-aggregate-verdict proof landed; premise retention, dispatch and the conformance controls are pending |
| TC-006 | Produce and strictly read canonical protocol results | Property | P0 | FR-006, FR-031, FR-032, FR-033 | 🚧 Planned |
| TC-007 | Project canonical protocol results to E02 | Property | P0 | FR-007, FR-034, FR-035 | 🚧 Planned |
| TC-008 | Reproduce results under equivalent presentations | Property | P0 | NFR-001, FR-016, FR-017, NFR-008 | 🚧 Planned — reproduction-record slice executed (12 tests, chunk C-31); equivalent-presentation permutations and the full one-axis mutation matrix remain planned |
| TC-009 | Enforce every protocol resource boundary | Property | P0 | NFR-002, FR-003, FR-016, FR-017, FR-025 | 🚧 Planned — the count, allocation and deadline guards landed in `tests/tc_009_bounds.rs` (18 tests, chunk C-29); the site inventory and its bidirectional reconciliation remain planned for C-30, and A's accumulation discipline is mirrored here rather than read from A |
| TC-010 | Preserve Rust and shared assurance containment | Static | P0 | NFR-003, FR-009, FR-017 | 🚧 Planned |
| TC-011 | Return typed refusals without text parsing | Property | P0 | FR-008, FR-014, FR-016, FR-036, FR-037, FR-038, FR-039, FR-040, NFR-008 | 🚧 Planned — backs FR-008-AC-1..AC-8 and the five atomic parts FR-036..FR-040; FR-008-AC-9 is unbacked. The envelope, the code catalog and the repository-wide code audit landed; AC-3 and AC-6 are pending catalog selection, and AC-5 and the policy halves of AC-4 and AC-8 are pending the cause taxonomy |
| TC-012 | Run the local verification gate and refuse an undefined lane | Static | P0 | NFR-004 | 🚧 Planned |
| TC-013 | Resolve the version-lock manifest | Property | P0 | FR-009, FR-012, FR-016, FR-017, FR-018, NFR-006 | 🚧 Planned — B-owned fixtures only; runnable without any external producer delivery |
| TC-014 | Apply the consumer identity discipline | Property | P0 | FR-011, FR-015, FR-017 | 🚧 Planned — B-owned fixtures only; runnable without any external producer delivery |

## Test Execution Summary

Row counts, recomputed from the tree. Every matrix row above is counted exactly
once, and `Blocked` means unexecutable — neither passing nor failing.

Engine-measured binding, which is the figure to read for coverage, is 37 of 322
rows (11%) — see **Engine-bound** above. The table below counts matrix rows and
their planned state; it does not measure binding.

| Category | Total | Passed | Failed | Blocked | Planned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Functional requirement rows | 32 | 0 | 0 | 0 | 32 |
| Domain object requirement rows | 9 | 0 | 0 | 0 | 9 |
| Non-functional requirement rows | 8 | 0 | 0 | 0 | 8 |
| Integration rows | 4 | 0 | 0 | 3 | 1 |
| Test case rows | 14 | 0 | 0 | 0 | 14 |
| Overall | 67 | 0 | 0 | 3 | 64 |

The Integration count is four because IT-001 carries a separate version 1 and version 2
row; the two contracts may not substitute for each other, so one row cannot
report both. Three of the four are unexecutable: IT-001 version 2, IT-002 and IT-003.

### Owed acceptance criteria

`NFR-005` denominates on what each lane **owes**, not on what it executed, so a
fully blocked integration cannot score 100%. These totals follow that rule: the
denominator is every authored acceptance criterion, and an unexecutable
criterion is a zero inside it rather than a row removed from it.

Engine-measured binding, which is the figure to read for coverage, is 37 of 322
rows (11%) — see **Engine-bound** above. The table below counts matrix rows and
their planned state; it does not measure binding.

| Category | Total | Passed | Failed | Blocked | Planned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Criteria backed by a test case | 229 | 0 | 0 | 0 | 229 |
| Criteria backed only by an integration scenario | 12 | 0 | 0 | 12 | 0 |
| Criteria with no test case and no integration scenario | 47 | 0 | 0 | 0 | 47 |
| Overall owed criteria | 288 | 0 | 0 | 12 | 276 |

The 12 unexecutable criteria are FR-001-AC-5, FR-002-AC-5, FR-013-AC-4,
FR-018-AC-3, FR-018-AC-5, FR-018-AC-6, FR-018-AC-8, NFR-007-AC-3, NFR-007-AC-4,
NFR-007-AC-5 and NFR-007-AC-7 — eleven whose backing scenarios cannot execute
under the declared gate because the `quire.compiled-protocol/2` handoff is not
reachable from a clean checkout; B pins A's promoted revision `04254a3` and the
scenarios do execute when `QUIRE_PROTOCOL_A_V2_HANDOFF` names the published
bundle — and FR-014-AC-6, waiting on IT-003 and Engineering Assurance #34.

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
