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
  - { target: ix://agent-ix/quire-protocol/FR-042, type: references }
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

This matrix tracks the first Rust implementation slice, including executed owner
paths and work that remains planned or blocked. The accepted frozen source baseline, F consumer correction, Engineering
Assurance producer, license, implementation plan, and retained corpus remain
explicit gates.

The repository carries 42 functional requirements, 8 non-functional
requirements, 3 integration tests and 15 test cases, and 294 acceptance criteria
across the 42 FRs and the 8 NFRs. The functional set divides into 9 aggregate or
behavioural parents (FR-001..FR-009), 9 object requirements (FR-010..FR-018),
and 24 atomic behavioural parts (FR-019..FR-042) that the parents aggregate
through `contains`. FR-003 through FR-008 now state their obligations once each
through those parts; the parent row and the part rows are therefore two views of
one obligation and must not be added together as separate work.

The 294 counts rows of a `## Acceptance Criteria` table on an FR or an NFR, and
nothing else. `quire properties` reports 314 criteria over the same tree: the
extra 20 are the `StR-0NN-VC-n` validation criteria of StR-001 through StR-005,
which are stakeholder validation rather than FR/NFR acceptance and which this
matrix carries no table for. Every one of the 294 appears in the engine's 314.
NFR-005's five metrics are outside both figures, because that requirement states
metrics and no acceptance criteria.

### What each status word in this matrix means

The five words below are not interchangeable, and no total mixes them.

- **Test-backed** — the criterion's `Verification` cell names at least one
  authored test case (`TC-0NN`). 239 of 294 criteria.
- **Integration-backed** — the cell names only an integration scenario
  (`IT-0NN`) and no test case. 8 of 294 criteria.
- **Unbacked** — the cell names no test case and no integration scenario: it
  verifies by `Inspection` or `Analysis` only. 47 of 294 criteria (37
  Inspection, 10 Analysis). An unbacked criterion is not covered by a test and
  this matrix writes `none` in its Test Cases cell rather than a test-case id
  that would imply otherwise. TC-015 is the highest authored test case; no row
  cites a higher id.
- **Engine-bound** — a compiler-checked trace symbol actually resolves to the
  criterion. This is the only one of the five words that is *measured* rather
  than read off a `Verification` cell, and it is much smaller than the others:

  ```
  $ quire coverage --scope .
  Coverage: 109/329 rows backed (33%)
  ```

  At the 2026-09-14 Task-001 checkpoint, 109 rows are engine-backed.
  NFR-001 is 3 of 3, NFR-002 is 4 of 4, NFR-004 is 10 of 10, and 11 of the 15
  test-case rows have at least one bound symbol. The remaining row-level gaps are enumerated by the
  same machine output and remain implementation work; this figure is not a
  qualification claim.

  The same run reports a binding census, which is related to `NFR-003-AC-5`
  without being the measurement that criterion asks for:

  ```
  rust: 237/237/237 bound/tagged/candidates (100% read; 100% authored; forms:
  rust-trace-attribute, rust-trace-line, rust-comment-id, rust-test-name-id,
  rust-doc-comment-id)
  ```

  Every one of the 237 Rust test symbols carries a marker and every marker is
  read. **`bound` counts symbols, not obligations** — a test carrying five ids is
  one bound candidate, and a test carrying only `Task-009` is a bound candidate
  too. `NFR-003-AC-5` asks for a binding "to each obligation it claims", which is
  a per-relation property this figure does not measure: the next section records
  version 2 support checks carrying `Task-012` without `IT-001`, and
  `tests/it001_native_handoff.rs` under-tagged at 24 traces over 4 criteria. All
  of that is consistent with 237/237. Read this line as "every test says
  something", never as "every claim is bound".

  Separately, and verified by reading the source rather than by this line: every
  marker is a **bare** `#[trace(...)]` over an imported `ix_trace_rs::trace`.
  The census cannot establish that on its own — `bound` is equally consistent
  with binding through the `rust-comment-id`, `rust-test-name-id` or
  `rust-doc-comment-id` forms it lists. That distinction matters because the
  path-qualified `#[ix_trace_rs::trace(...)]` compiles, expands identically, and
  binds nothing, because the marker pattern is anchored on the attribute name.

  That hazard was measured, not imagined. Path-qualifying all 198 markers in a
  prior-corpus throwaway worktree took this repository from **37 backed rows to 8**, with no
  warning, no compile error and a still-green gate. Note what the census does
  *not* do in that state: it reports 198 tagged and **134 bound**, not zero,
  because `rust-test-name-id` above is a fallback that recovers a binding from a
  `tc_NNN_` function name after the attribute has been defeated. So a reader who
  path-qualifies a marker and checks the census for a collapse to zero sees 68%
  and concludes nothing broke, while 29 rows have silently gone. The row count
  is the number that moves; the census percentage is not.

  `NFR-003-AC-5` is itself **not** engine-bound, and the audit that would *keep*
  the census at 100% — the thing that fails when someone adds an untagged test or
  path-qualifies a marker — is not written here.

  The reason is narrow, and it is worth stating narrowly. AC-5's only declared
  verification is `Test (TC-010)`; TC-010 is Task-011's; Task-011 is `blocked`.
  Its own subtask for TC-010 ends "add no local scanner", so a scanner written
  here would not discharge the criterion even if it worked. An earlier revision
  of this paragraph reached instead for NFR-003's Verification clause forbidding
  a local replacement. That clause is about the shared Rust-native
  *language-boundary* checker, which verifies AC-3, not AC-5 — and the same
  sentence names Quire coverage as a permitted method. Stretching a real
  prohibition to cover something it does not reach is not a stronger reason for
  waiting; it is a weaker one, because it can be checked and found wrong.

  Nor is TC-010 one upstream publish away. Task-011 `depends_on` Task-008,
  Task-009 and Task-010, and its first subtask freezes five external revisions
  at once: Engineering Assurance issue 34, E02, the Quoin interface, Quoin
  containment (`agent-ix/quire-research#62`) and the shared Rust checker
  (`agent-ix/quire-research#64`). Naming only #64 would make the block read as
  nearly lifted.

  Note also what `ix-trace-rs` does and does not check, since "compiler-checked"
  is easy to over-read. The macro validates argument **shape** only: it rejects
  a non-literal or a malformed id at compile time, and it explicitly does not
  know which kinds exist or whether an id appears in any matrix. So deleting a
  `#[trace(...)]` outright, or misspelling `FR-002-AC-2` into a well-shaped and
  nonexistent `FR-002-AC-9`, compiles clean. Only `quire coverage` catches
  either, and no gate command asserts the census.

  FR-002 is 3 of 5, and the three are `FR-002-AC-2`, `FR-002-AC-3` and
  `FR-002-AC-4`. AC-2 joined them with chunk C-14, from `src/link/mod.rs` and
  `src/link/interleaving.rs`; AC-3 and AC-4 are C-12's, from `src/link/mod.rs`.

  **Every FR-002 criterion binding is in `src/link/mod.rs` and
  `src/link/interleaving.rs`. None is in `tests/tc_002_identities.rs`** — its six
  markers are `Task-002`, `FR-011-AC-2` and `FR-011-AC-3`, and it names no FR-002
  criterion at all. A revision of this paragraph credited AC-3 and AC-4 partly to
  that file. Crediting FR-002 to a tag `tests/tc_002_identities.rs` does not
  carry is the exact defect the TC-002 row below records: an FR-002 criterion was
  once counted off a tag in that file, the owner ruled the criterion unbindable,
  and the tag became `Task-002`. Naming the file again as FR-002 evidence is how
  someone re-adds the tag.

  A revision of this paragraph described the head at `1386ed1` and reported
  "37 rows" and "FR-002 3 of 5". Measured at that head, the figures were **36
  of 322** and **FR-002 2 of 5**: AC-2 had no tag anywhere until C-14 landed,
  which is what took the total to 37 and FR-002 to 3. The paragraph was written
  from the work in flight rather than from a run, and landing C-14 made it
  accidentally true. The figures above are a run over the head this file is
  committed at.

  `FR-002-AC-1` stays unbound until #4 resolves. It had been bound once:
  `tests/it001_native_handoff.rs` tagged it, the owner ruled the criterion
  unresolved until runtime instance identities exist, and the tag went. The
  figure fell because the measurement got more honest, not because anything
  stopped working — that test is `#[ignore]`d, so the criterion was never
  checked by a run in the first place. `FR-002-AC-5` now executes through the
  published version 2 handoff but remains engine-unbound at criterion level;
  the lane binds its owning `IT-001` scenario instead.

  The 239 test-backed criteria and 109 engine-bound rows are not contradictory:
  the two words have different denominators, and the definitions above say so.
  **Read 109/329 as the coverage figure.** "Test-backed" says a criterion has
  somewhere to be tested; "engine-bound" says a trace reaches a repository row.

  The gap closes by tagging, never by editing this table. A trace attribute is
  the only thing that binds a test to a criterion, so a criterion a test
  exercises without naming it stays unbound however this matrix describes it.
  Tagging `tc_004`, `tc_006`, `tc_002` and `tc_008` at criterion level took
  coverage from 13 rows to 24 without changing one line of test logic, and
  removing one criterion a test could not check took it back to 23.

  Two kinds of unbound symbol remain, and they are not the same thing:

  - **Deliberately unbound.** The superseded 24-test pre-delivery corpus was
    retired when A's published manifest became the sole version 2 scenario
    authority. Of the 17 tests in `tests/it001_v2_handoff.rs`, 6 execute IT-001
    behaviour and carry `IT-001`, 2 independently close TC-001's baseline and
    numeric controls, and the remaining 9 exercise corpus inventory or B's
    integrity mechanism and carry only `Task-012`. Tagging support checks as
    integration evidence would still inflate coverage, so those 9 remain
    deliberately unbound to a test case or criterion.
  - **Still under-tagged.** Tests that exercise a criterion they do not name.
    `tests/it001_native_handoff.rs` is the largest remaining case, at 24 traces
    over 4 criteria.

- **Unexecutable** — a backing exists but cannot be run, so the criterion is
  neither passing nor failing. 1 of 294 criteria,
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
may not substitute for each other. The version 2 row executes under the default
gate; its broader version 1 regression subscenario SC-06a remains planned under
Task-009.

A has published a `/2` handoff, and B runs it on every default local gate:
`tests/it001_v2_handoff.rs`
admits and links A's offered bytes against an expectation built only from A's
independent selection sidecar and the original inputs, and submits all 28 cases
of A's mutation manifest. On the revision named below, the base offer is admitted
and every mutation refuses with the identity A declares for it.

The exact pin is `quire-spec-language`
`f1700a9264d6d3bcdd07e0f77b70f3dae9ed4c07`, a post-`v0.2.0` revision. A
exports `PUBLISHED_HANDOFF` plus every member-name
and manifest-format constant. B loads that owner path directly, verifies all
109 published members, refuses unlisted or corrupt members, admits the base,
checks both cross-version directions, submits all 28 declared mutations, and
probes all 13 `Dimension::ALL` budget members. No environment selector or
vendored mirror remains, and the lane reports `17 passed; 0 ignored`.

`FR-009`'s entry-status table records that every version-lock entry is
`unresolved` today. IT-002 and IT-003 declare FR-009 preconditions, so every one
of their scenarios is unexecutable. IT-001 declares no manifest precondition, so
its version 1 lane SC-01..SC-05 remains planned rather than unexecutable. The
version 2 lane is executable from the exact `f1700a9` pin.

## Functional Requirement Coverage

Parent and atomic rows are both listed. FR-010..FR-018 carry `object:` and are
in Domain Object Coverage instead.

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- |
| FR-001 | FR-001-AC-1, FR-001-AC-2, FR-001-AC-3, FR-001-AC-4, FR-001-AC-5 | TC-001 (AC-1..AC-4); IT-001 (AC-4, AC-5) | 🚧 In progress — 4 of 5 criteria are engine-backed. Task-001 and TC-001 close AC-1..AC-4 on the pinned default-running `/2` handoff; AC-5 remains planned because IT-001 SC-06a still owes the complete two-revision version 1 comparison under Task-009 |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-5 | TC-002 (AC-1..AC-4); IT-001 (AC-5) | 🚧 Planned — 5 of 5 criteria backed, 0 unbacked. FR-002-AC-5 executes through A's published version-2 handoff. AC-2 executes over the existing static graph. AC-1, the runtime-binding portion of AC-3, and the FR-287/FR-288/FR-216/FR-217/FR-262 completion of AC-4 remain Task-002 implementation work on issue #4; their former owner blockers are delivered by QSpec PRs #41/#42 and QObs PR #20. |
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
| FR-031 | FR-031-AC-1..AC-6 | TC-006 (AC-1..AC-6), TC-008 (AC-5) | 🚧 In progress — 6 of 6 criteria have direct tests; Task-006 remains open for its other canonical-result and refusal deliverables |
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
| FR-042 | FR-042-AC-1..AC-6 | TC-015 (AC-1..AC-6) | ✅ Implemented — all six criteria carry direct traced tests; the current engine reports 3 of 6 criterion rows bound and no FR-042 unbacked or untracked record, so the direct trace inventory and engine rollup remain stated separately |

Behavioural totals across these 33 rows: 183 acceptance criteria, of which 168
are backed by a test case, 2 are backed only by an integration scenario
(FR-002-AC-5 passes; FR-001-AC-5 remains planned pending SC-06a), and 13 are
unbacked — 9 by Inspection and 4 by Analysis.
168 + 2 + 13 = 183.

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
| FR-013 | value_object | FR-013-AC-1..AC-8 | TC-001 (AC-1, AC-2, AC-3, AC-5, AC-6); IT-001 (AC-1..AC-4, AC-6); none for FR-013-AC-7, FR-013-AC-8 | 🚧 Planned — 5 of 8 criteria test-backed; unbacked: FR-013-AC-7 (Inspection), FR-013-AC-8 (Analysis). FR-013-AC-4 now executes in both ordered directions through the explicit `/1` and `/2` entry points and passes with distinct typed causes. |
| FR-014 | value_object | FR-014-AC-1..AC-8 | TC-011 (AC-1, AC-2, AC-4, AC-5, AC-7); IT-003 (AC-5, AC-6); none for FR-014-AC-3, FR-014-AC-8 | 🚧 Planned — 5 of 8 criteria test-backed; unbacked: FR-014-AC-3 (Analysis), FR-014-AC-8 (Inspection); unexecutable: FR-014-AC-6, whose sole backing is IT-003, blocked on Engineering Assurance #34 |
| FR-015 | value_object | FR-015-AC-1..AC-5 | TC-014 (AC-2, AC-3, AC-4); TC-001 (AC-2); none for FR-015-AC-1, FR-015-AC-5 | 🚧 Planned — 3 of 5 criteria test-backed; unbacked: FR-015-AC-1, FR-015-AC-5 (Inspection). The production and version 2 digest enums are two different vocabularies sharing one member name; FR-015 records the divergence as unreconciled |
| FR-016 | enumeration | FR-016-AC-1..AC-8 | TC-009 (AC-2..AC-7); TC-011 (AC-5); TC-008 (AC-6); TC-013 (AC-8); IT-001 (AC-2); none for FR-016-AC-1 | 🚧 Planned — 7 of 8 criteria test-backed; unbacked: FR-016-AC-1 (Inspection); 0 unexecutable |
| FR-017 | entity | FR-017-AC-1..AC-8 | TC-013 (AC-2); TC-008, TC-009 (AC-5); TC-010 (AC-7); TC-014 (AC-8); IT-002, IT-003 (AC-2); none for FR-017-AC-1, FR-017-AC-3, FR-017-AC-4, FR-017-AC-6 | 🚧 Planned — 4 of 8 criteria test-backed; unbacked: FR-017-AC-1, FR-017-AC-3, FR-017-AC-4 (Inspection), FR-017-AC-6 (Analysis); 0 unexecutable |
| FR-018 | entity | FR-018-AC-1..AC-9 | TC-013 (AC-1, AC-3, AC-5, AC-6, AC-8); none for FR-018-AC-2, FR-018-AC-4, FR-018-AC-7, FR-018-AC-9 | 🚧 Planned — 5 of 9 criteria test-backed; unbacked: FR-018-AC-2, FR-018-AC-4, FR-018-AC-7, FR-018-AC-9. The retired pre-delivery corpus is no longer credited to IT-001, and A's published `MutationManifest` is producer-owned rather than an FR-018 instance. |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
| --- | --- | --- | --- |
| NFR-001 | Property and metamorphic testing | TC-008 (AC-1..AC-3) | 🚧 In progress — 3 of 3 criteria are engine-backed. The canonical result now retains the complete reproduction tuple; map order, locale-isolated child processes, allocation layout, permitted independent arrival order, authored causal order, and the named result-identity mutation matrix execute. End-to-end E02 and observation/temporal integration permutations remain owned by Task-007 and Task-010, so this is not yet a plan-complete claim |
| NFR-002 | Property, mutation, and source-bound inventory inspection | TC-009 (AC-1..AC-4) | ✅ Implemented on current owner paths — 4 of 4 criteria engine-backed. The production inventory reconciles source sites bidirectionally; delete, duplicate, orphan and missing-boundary mutants fail; every production site has below/at/above evidence; static and elapsed-time accounting are separate; production deadline tests exercise before/after checks and honest overshoot; action-capable assessment doubles record zero calls. Projection-path coverage remains contingent on Task-007 |
| NFR-003 | Shared Rust language-boundary check, static analysis, architecture inspection, Rust review, Quire coverage, gap analysis | TC-010 (AC-1..AC-6); IT-003 (AC-4, AC-6) | 🚧 Planned — 6 of 6 criteria test-backed, 0 unbacked, 0 unexecutable |
| NFR-004 | Cargo metadata inspection, conditional-compilation inventory, enumerated gate execution over one unchanged head, per-criterion negative controls | TC-012 (AC-1..AC-10) | ✅ Implemented — 10 of 10 criteria are engine-backed by ten executable TC-012 tests. The actual documented commands, manifest and four conditional sites are checked alongside isolated command/risk, serialization, warning, feature/lane, target and evidence-state mutations. Both real compiler warning-channel controls and the undeclared-feature control refuse as specified. NFR-004-AC-2 remains inspection-method but test-case-backed. |
| NFR-005 | Static inspection and Quire coverage over the integration lanes, denominated on owed rather than executed scenarios | IT-002, IT-003 | 🚧 Planned — this requirement states five metrics and no acceptance criteria, so it contributes 0 to the 294. Its denominator is the 22 scenarios IT-002 and IT-003 owe (12 + 10); all 22 are unexecutable, so every metric scores 0 of 22 rather than a vacuous 100% |
| NFR-006 | Cargo manifest, lockfile and rights-record inspection with per-criterion negative controls; delivery-surface tests on B-owned fixtures | TC-013 (AC-6, AC-7, AC-8); none for NFR-006-AC-1..AC-5 | 🚧 Planned — 3 of 8 criteria test-backed; unbacked: NFR-006-AC-1, NFR-006-AC-2, NFR-006-AC-3, NFR-006-AC-4, NFR-006-AC-5 (Inspection); 0 unexecutable. The current lockfile has 132 packages total, all 131 external packages carry a checksum or exact git revision, and `DEPENDENCIES.md` agrees with all four git-sourced revisions. |
| NFR-007 | Public-surface inspection, two-revision surface analysis, and IT-001 cross-version and inherited-axis scenarios | IT-001 (AC-3, AC-4, AC-5, AC-7); none for NFR-007-AC-1, NFR-007-AC-2, NFR-007-AC-6, NFR-007-AC-8 | 🚧 Planned — 0 of 8 criteria test-backed and 4 integration-backed. AC-3 and AC-4 pass: both ordered cross-version submissions refuse with distinct typed causes, and neither cause collides with any of A's 28 published mutation outcomes. AC-7 still owes the earlier-path comparison for every inherited axis; AC-5's complete SC-06a regression rerun remains planned. Unbacked: AC-1, AC-2, AC-8 (Inspection), AC-6 (Analysis). |
| NFR-008 | Static site counts over the repository's own sources, a rendering-replacement mutation, a captured-stream control, an accessor compilation control, and reproduction comparison | TC-011 (AC-1, AC-2, AC-6); TC-008 (AC-7); none for NFR-008-AC-3, NFR-008-AC-4, NFR-008-AC-5, NFR-008-AC-8 | 🚧 Planned — 4 of 8 criteria test-backed; unbacked: NFR-008-AC-3, NFR-008-AC-4, NFR-008-AC-5, NFR-008-AC-8 (Inspection); 0 unexecutable |

Non-functional totals across these 8 rows: 47 acceptance criteria, of which 30
are backed by a test case, 4 are backed only by an integration scenario
(NFR-007-AC-3 and AC-4 pass; AC-5 and AC-7 remain planned), and 13 are unbacked —
12 by Inspection and 1 by Analysis. 30 + 4 + 13 = 47. NFR-005 adds no criterion to that count, because it
states five metrics and no acceptance criteria, and is denominated on owed
scenarios instead.

Across all three coverage tables the three per-criterion figures reconcile to
the owed total: 168 + 41 + 30 = 239 test-backed, 2 + 2 + 4 = 8
integration-backed, and 13 + 21 + 13 = 47 unbacked, summing to the 294 authored
acceptance criteria. Of the integration-backed set, 4 pass, 3 remain planned,
and 1 is unexecutable.

## Integration Coverage

| Integration Test | Boundary | Requirements | Status |
| --- | --- | --- | --- |
| IT-001 version 1 | Native compiler `quire.compiled-protocol/1` artifact to protocol library | FR-001, FR-002, FR-013, FR-016, NFR-002 | 🚧 Planned — SC-01..SC-05, 5 owed scenarios; fixture lane `#[ignore]`. IT-001 declares no FR-009 manifest precondition, so this lane is planned rather than unexecutable |
| IT-001 version 2 | Native compiler version 2 artifact to protocol library | FR-001-AC-5, FR-002-AC-5, FR-013-AC-4, NFR-002, NFR-007 | 🚧 In progress — 4 of 5 owed scenarios execute and pass under the default local gate: SC-06's `/2` admission and both cross-version directions, SC-07, SC-08, and SC-09. The exact `f1700a9` pin supplies `PUBLISHED_HANDOFF`; all 109 checksummed members, the base admission, all 28 declared typed mutations, independent accepted-baseline substitutions, exact and invalid numeric controls, both cross-version refusals, and all 13 exact/one-short/fresh budget dimensions execute in 17 non-ignored tests. SC-06a's complete earlier-version SC-01..SC-05 regression comparison remains planned under Task-009. |
| IT-002 | Observation and temporal facts to protocol evaluator/result | FR-003, FR-004, FR-006, FR-009, FR-017, NFR-002, NFR-005 | 🚧 Planned — SC-01, SC-02, SC-03, SC-04, SC-05a..SC-05d, SC-06, SC-07a, SC-07b, SC-08, 12 owed scenarios, all 12 unexecutable and scoring 0 of 12; blocked on accepted F/E producer contracts and on every FR-009 entry being `unresolved` |
| IT-003 | Protocol result through Engineering Assurance, E02, and Quoin | FR-006, FR-007, FR-009, FR-014, FR-017, FR-039, FR-040, NFR-002, NFR-003, NFR-005 | 🚧 Planned — SC-01..SC-07, SC-08a, SC-08b, SC-09, 10 owed scenarios, all 10 unexecutable and scoring 0 of 10; blocked on Engineering Assurance #34 |

Owed integration scenarios: 32 across the three integration tests (IT-001 10,
IT-002 12, IT-003 10). 4 pass, 6 are planned, and 22 are unexecutable. No
scenario is reported as failing. IT-001 version 2 contributes the four passing
scenarios and its SC-06a regression comparison is one of the six planned.

## Test Case Summary

`Traces To` names every requirement whose acceptance criteria cite the test case,
parents and atomic parts alike. A test case whose Traceability section does not
yet name every requirement listed here is a back-edge gap in that test case, not
a coverage claim in this matrix.

| Test ID | Title | Type | Priority | Traces To | Status |
| --- | --- | --- | --- | --- | --- |
| TC-001 | Admit only the selected protocol profile | Property | P0 | FR-001, FR-013, FR-015 | ✅ Implemented — 17 default-running version 2 handoff tests cover exact admission/linking, all owner-declared mutations, independent baseline selection, exact and invalid numeric boundaries, both explicit cross-version refusal directions and every resource dimension. It adds no local decoder or refusal vocabulary; FR-001-AC-5 remains IT-001/Task-009 work |
| TC-002 | Link and discriminate finite protocol structure | Property | P0 | FR-002 | 🚧 Planned — the existing static linker executes in 28 tests in `src/link/mod.rs`, 4 in `src/link/interleaving.rs` and 6 in `tests/tc_002_identities.rs`. It proves AC-2 and partial AC-3/AC-4 without claiming AC-1. Task-002 now adds the delivered-owner completion: componentwise FR-288 identity, complete typed semantic consistency content, imported FR-287 runtime subjects, exact FR-217 role classification, five-component FR-216/FIFO identity and FR-262 replay/contradiction handling. The expanded one-axis inventory and exact post-change counts will replace this planned note only after the tests exist and run. |
| TC-003 | Assess compensation, retry, commit, and recovery | Property | P0 | FR-003, FR-019, FR-020, FR-021, FR-022, FR-023, FR-024, FR-025 | 🚧 Planned — backs the parent and all seven atomic parts; its Traceability section names only FR-003 today |
| TC-004 | Assess finite global conformance | Property | P0 | FR-004, FR-026, FR-027, FR-028, FR-041 | 🚧 Planned (FR-004-AC-3 landed; AC-1/AC-2/AC-4 pending the batch and incremental evaluators) |
| TC-005 | Separate protocol claim dispositions | Property | P0 | FR-005, FR-029, FR-030 | 🚧 Planned — the six-disposition record and its no-aggregate-verdict proof landed; premise retention, dispatch and the conformance controls are pending |
| TC-006 | Produce and strictly read canonical protocol results | Property | P0 | FR-006, FR-031, FR-032, FR-033 | 🚧 Planned |
| TC-007 | Project canonical protocol results to E02 | Property | P0 | FR-007, FR-034, FR-035 | 🚧 Planned |
| TC-008 | Reproduce results under equivalent presentations | Property | P0 | NFR-001, FR-016, FR-017, NFR-008 | 🚧 In progress — reproduction input refusal and identity tests, canonical-result provenance retention, map/locale/allocation/arrival-order permutations, authored-order controls, and the complete named result-identity mutation matrix execute. Task-007 and Task-010 still owe projection and producer-integration permutations |
| TC-009 | Enforce every protocol resource boundary | Property | P0 | NFR-002, FR-003, FR-016, FR-017, FR-025 | ✅ Implemented on current owner paths — 33 core inventory/guard tests plus bounded admission, link, compensation, conformance, claim, result, Contract-IR and reproduction cases execute. New production sites must enter the bidirectional inventory and boundary registry or the mutation ratchet fails |
| TC-010 | Preserve Rust and shared assurance containment | Static | P0 | NFR-003, FR-009, FR-017 | 🚧 Planned |
| TC-011 | Return typed refusals without text parsing | Property | P0 | FR-008, FR-014, FR-016, FR-036, FR-037, FR-038, FR-039, FR-040, NFR-008 | 🚧 Planned — backs FR-008-AC-1..AC-8 and the five atomic parts FR-036..FR-040; FR-008-AC-9 is unbacked. The envelope, the code catalog and the repository-wide code audit landed. C-05 now preserves A's typed error and stable code without parsing text, but deliberately carries no FR-008 trace: it cannot form the complete envelope while AC-3/AC-6 await an adopted catalog selection (upstream FR-270) and AC-5 plus the policy halves of AC-4/AC-8 await the typed cause contract (upstream FR-272) |
| TC-012 | Run the local verification gate and refuse an undefined lane | Static | P0 | NFR-004 | ✅ Implemented — ten executable tests bind all ten criteria to the exact repository snapshot and thirteen stable defect identities; no test is ignored or environment-gated. |
| TC-013 | Resolve the version-lock manifest | Property | P0 | FR-009, FR-012, FR-016, FR-017, FR-018, NFR-006 | 🚧 Planned — B-owned fixtures only; runnable without any external producer delivery |
| TC-014 | Apply the consumer identity discipline | Property | P0 | FR-011, FR-015, FR-017 | 🚧 Planned — B-owned fixtures only; runnable without any external producer delivery |
| TC-015 | Produce, read and map canonical protocol results | Integration | P0 | FR-042-AC-1..FR-042-AC-6 | ✅ Implemented — 12 executable tests cover canonical production/strict reading, complete typed views, all four independent axes, Boolean/non-value mapping, identity/lineage mutations, schema pins, reproduction and exact/one-short resource limits |

## Test Execution Summary

Row counts, recomputed from the tree. Every matrix row above is counted exactly
once, and `Blocked` means unexecutable — neither passing nor failing.

Engine-measured binding at the current implementation checkpoint is 109 of 329
rows (33%) — see **Engine-bound** above. The table below counts matrix rows and
their planned state; it does not measure binding.

| Category | Total | Passed | Failed | Blocked | Planned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Functional requirement rows | 33 | 1 | 0 | 0 | 32 |
| Domain object requirement rows | 9 | 0 | 0 | 0 | 9 |
| Non-functional requirement rows | 8 | 2 | 0 | 0 | 6 |
| Integration rows | 4 | 0 | 0 | 2 | 2 |
| Test case rows | 15 | 4 | 0 | 0 | 11 |
| Overall | 69 | 7 | 0 | 2 | 60 |

The Integration count is four because IT-001 carries a separate version 1 and version 2
row; the two contracts may not substitute for each other, so one row cannot
report both. IT-002 and IT-003 are unexecutable. The two IT-001 rows are planned:
the version 1 lane has not executed its generated fixture, and the version 2
lane passes four scenarios while SC-06a remains planned.

### Owed acceptance criteria

`NFR-005` denominates on what each lane **owes**, not on what it executed, so a
fully blocked integration cannot score 100%. These totals follow that rule: the
denominator is every authored acceptance criterion, and an unexecutable
criterion is a zero inside it rather than a row removed from it.

Engine-measured binding at the current implementation checkpoint is 109 of 329
rows (33%) — see **Engine-bound** above. The table below counts matrix rows and
their planned state; it does not measure binding.

| Category | Total | Passed | Failed | Blocked | Planned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Criteria backed by a test case | 239 | 24 | 0 | 0 | 215 |
| Criteria backed only by an integration scenario | 8 | 4 | 0 | 1 | 3 |
| Criteria with no test case and no integration scenario | 47 | 0 | 0 | 0 | 47 |
| Overall owed criteria | 294 | 28 | 0 | 1 | 265 |

The four passing integration-only criteria are FR-002-AC-5, FR-013-AC-4,
NFR-007-AC-3 and NFR-007-AC-4. FR-001-AC-5, NFR-007-AC-5 and NFR-007-AC-7 remain
planned because their complete version 1 regression or inherited-axis comparison is
not established by Task-012. The one unexecutable criterion is FR-014-AC-6,
waiting on IT-003 and Engineering Assurance #34.

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
test-case id above TC-015, and none is invented to close one of these gaps.
