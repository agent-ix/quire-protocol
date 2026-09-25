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

The repository carries 44 functional requirements, 8 non-functional
requirements, 3 integration tests and 19 test cases, and 315 acceptance criteria
across the 44 FRs and the 8 NFRs. The functional set divides into 9 aggregate or
behavioural parents (FR-001..FR-009), 9 object requirements (FR-010..FR-018),
24 atomic behavioural parts (FR-019..FR-042) that the parents aggregate
through `contains`, and 2 independent behavioural requirements (FR-301, FR-302)
that extend FR-005 and FR-006's claim dispatch and result-wire obligations
rather than being decomposed from them. FR-003 through FR-008 now state their
obligations once each through those parts; the parent row and the part rows are
therefore two views of one obligation and must not be added together as
separate work.

The 315 counts rows of a `## Acceptance Criteria` table on an FR or an NFR, and
nothing else. `quire properties` reports 335 criteria over the same tree: the
extra 20 are the `StR-0NN-VC-n` validation criteria of StR-001 through StR-005,
which are stakeholder validation rather than FR/NFR acceptance and which this
matrix carries no table for. NFR-005's five metrics are outside that figure;
its six acceptance criteria are included.

### What each status word in this matrix means

The five words below are not interchangeable, and no total mixes them.

- **Test-backed** — the criterion's `Verification` cell names at least one
  authored test case (`TC-0NN`). 275 of 315 criteria.
- **Integration-backed** — the cell names only an integration scenario
  (`IT-0NN`) and no test case. 8 of 315 criteria.
- **Unbacked** — the cell names no test case and no integration scenario: it
  verifies by `Inspection` or `Analysis` only. 32 of 315 criteria (28
  Inspection, 4 Analysis). An unbacked criterion is not covered by a test and
  this matrix writes `none` in its Test Cases cell rather than a test-case id
  that would imply otherwise. TC-148 is the highest authored test case; no row
  cites a higher id.
- **Engine-bound** — a compiler-checked trace symbol actually resolves to the
  criterion. This is the only one of the five words that is *measured* rather
  than read off a `Verification` cell, and it is much smaller than the others:

  ```
  $ quire coverage --scope .
  Coverage: 175/354 rows backed (49%)
  ```

  At the 2026-09-18 checkpoint, 175 rows are engine-backed; the denominator
  includes FR-301 and FR-302's ten planned, still-unbound criteria.
  NFR-001 is 3 of 3, NFR-002 is 4 of 4, NFR-004 is 10 of 10, and 13 of the 19
  test-case rows have at least one bound symbol. The remaining row-level gaps are enumerated by the
  same machine output and remain implementation work; this figure is not a
  qualification claim.

  The same run reports a binding census, which is related to `NFR-003-AC-5`
  without being the measurement that criterion asks for:

  ```
  rust: 266/266/272 bound/tagged/candidates (97% read; 97% authored; forms:
  rust-trace-attribute, rust-trace-line, rust-comment-id, rust-test-name-id,
  rust-doc-comment-id)
  ```

  Every tagged Rust test symbol is read; six current candidates remain untagged
  and unbound outside Task-003. **`bound` counts symbols, not obligations** — a test carrying five ids is
  one bound candidate, and a test carrying only `Task-009` is a bound candidate
  too. `NFR-003-AC-5` asks for a binding "to each obligation it claims", which is
  a per-relation property this figure does not measure: the next section records
  version 2 support checks carrying `Task-012` without `IT-001`, and
  `tests/it001_native_handoff.rs` under-tagged at 24 traces over 4 criteria. All
  of that is consistent with the current census. Read this line as "every
  tagged test says something the engine can read", never as "every test is
  tagged" or "every claim is bound".

  Integration-only criteria remain a separately measured class. The installed
  traceability model cannot mint IT scenario IDs because they are bullet items,
  not table rows (`quire-rs#244`). FR-001-AC-5 and NFR-007-AC-3/AC-4/AC-7 are
  nevertheless criterion-bound directly on the current executable tests;
  SC-06a's two-revision result remains historical execution evidence rather
  than a persistent single-run source binding.

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

  The 275 test-backed criteria and 175 engine-bound rows are not contradictory:
  the two words have different denominators, and the definitions above say so.
  **Read 175/354 as the coverage figure.** "Test-backed" says a criterion has
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
  neither passing nor failing. 1 of 315 criteria,
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

Task-009's version-1 compatibility lane runs by default from A's committed,
owner-addressed handoff. Its 24 tests admit through the explicit version-1
intake, execute the local static linker, preserve the independent selection and
numeric/digest boundaries, and positively inventory the sole reader path.

Every protocol, temporal and observation noun this repository's requirements use
resolves to an `ix://agent-ix/quire-specification/FR-2NN` reference rather than a
local definition. QSpec #28/#29/#31/#40 closed the earlier identity, result,
channel, relationship, activation, completeness and boundary gaps. FR-236's
per-attempt versus aggregate outcome partition is the one current semantic
blocker; only the affected Task-003 mapping cases remain unexecutable.

IT-001 is split into two rows because version 1 and version 2 are separate contracts that
may not substitute for each other. Both rows execute under the default gate.
SC-06a additionally passed the identical historical 23-case version-1 corpus at
the actual Protocol revisions before and adding version 2.

A has published a `/2` handoff, and B runs it on every default local gate:
`tests/it001_v2_handoff.rs`
admits and links A's offered bytes against an expectation built only from A's
independent selection sidecar and the original inputs, and submits all 28 cases
of A's mutation manifest. On the revision named below, the base offer is admitted
and every mutation refuses with the identity A declares for it.

The exact pin is `quire-spec-language`
`51245876e0d406aeed3047c608ad478b53b19205`, a post-`v0.2.0` revision. A
exports `PUBLISHED_HANDOFF` plus every member-name
and manifest-format constant. B runs that owner's producer example from the pinned source (the committed copy omits dependency bytes), verifies all
listed members, refuses unlisted or corrupt members, admits the base,
checks both cross-version directions, submits all 28 declared mutations, and
probes all 13 `Dimension::ALL` budget members. No environment selector or
vendored mirror remains, and the lane reports `18 passed; 0 ignored`.

`FR-009` records status per exact contract. QSL, QObs, Engineering Assurance
and qverify E02 are pinnable; QObs is pinned at accepted owner PR #27 merge
`9240063`.
qverify's exact compatible consumer revision is `293e342`. The Quoin Protocol
scenario intake remains unresolved. Only scenarios requiring an unresolved
entry are unexecutable. IT-001's full-assessment path remains
subject to capability resolution; the version-1 lane and SC-06a exercise only
the declared legacy `/1` compatibility exception.

## Functional Requirement Coverage

Parent and atomic rows are both listed. FR-010..FR-018 carry `object:` and are
in Domain Object Coverage instead.

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- |
| FR-001 | FR-001-AC-1, FR-001-AC-2, FR-001-AC-3, FR-001-AC-4, FR-001-AC-5 | TC-001 (AC-1..AC-4); IT-001 (AC-4, AC-5) | ✅ Implemented — Task-001/TC-001 close AC-1..AC-4; Task-009 closes AC-5 with default explicit `/1` and `/2` paths, mutual typed refusal, and the passing SC-06a two-revision version-1 comparison. |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-5 | TC-002 (AC-1..AC-4); IT-001 (AC-5) | ✅ Implemented — PR #67 (`2a3ef19`) delivered the complete version-2 linker, componentwise FR-288 content, exact QObs runtime bindings, typed numeric/expression retention, FIFO premises and FR-262 replay/contradiction handling. IT-001 retains the separate native-integration obligation. |
| FR-003 | FR-003-AC-1, FR-003-AC-2, FR-003-AC-3, FR-003-AC-4, FR-003-AC-5, FR-003-AC-6, FR-003-AC-7 | TC-003 (AC-1..AC-7); TC-009 (AC-7) | ✅ Implemented — Task-003 connects the landed pure kernels to one graph-owned Engine. Exact QSL guard/retry/recovery inputs and reports plus QObs subjects/observation/position views are retained in replay-safe transactional registration, attempt/effect, selected-commit ordering and full/partial-recovery state; production exposes no recovery callback. Aggregate view of FR-019..FR-025. |
| FR-004 | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | TC-004 (AC-1..AC-4) | ✅ Complete — strict FR-300 activation facts derive the production model from the linked graph; the Engine synchronizes its compensation snapshot into that kernel; batch and incremental frontends share it. Aggregate view of FR-026..FR-028 and FR-041. |
| FR-005 | FR-005-AC-1, FR-005-AC-2, FR-005-AC-3, FR-005-AC-4, FR-005-AC-5 | TC-005 (AC-1..AC-5) | ✅ Complete — public Engine dispatch derives the actual strict activation conformance report, preserves independent claim rows and exact premises, and admits only the closed local conformance backend. Aggregate view of FR-029 and FR-030. |
| FR-006 | FR-006-AC-1..FR-006-AC-9 | TC-006 (AC-1..AC-9); TC-011 (AC-9) | ✅ Implemented — one atomic per-assessment Engine operation reuses exact conformance axes/completeness/global closure, derives truth and provenance from sealed dispatch/selection facts, privately produces and strictly reads canonical rows, then commits the AssessmentBundle. Aggregate view of FR-031..FR-033. |
| FR-007 | FR-007-AC-1, FR-007-AC-2, FR-007-AC-3, FR-007-AC-4, FR-007-AC-5, FR-007-AC-6 | TC-007 (AC-1..AC-6) | ✅ Complete — the total mapper, exact identity, owner strict read and four TC-007 symbols execute against exact qverify `293e342` / EA `9237ce5`; no local patch or target-contract mirror remains. Aggregate view of FR-034 and FR-035. |
| FR-008 | FR-008-AC-1, FR-008-AC-2, FR-008-AC-3, FR-008-AC-4, FR-008-AC-5, FR-008-AC-6, FR-008-AC-7, FR-008-AC-8, FR-008-AC-9 | TC-011 (AC-1..AC-9) | ✅ Implemented — the Task-017 foundation and Task-006 routing retain exact stages, prerequisites, axes, causes, source interiors, partial-output state and full Protocol resource usage. Static TC-011 inspection verifies the complete FR-036..FR-040 decomposition. |
| FR-009 | FR-009-AC-1..FR-009-AC-9 | TC-013 (AC-1..AC-9); TC-010 (AC-7); IT-001, IT-002, IT-003 (AC-1..AC-6) | 🚧 In progress — Task-013's resolver tests execute all nine criteria with deterministic bounded request-bound capabilities. Task-020's IT-002 SC-08 test resolves fifteen QObs owner contracts plus four versioned QSL native-temporal contracts, compiled protocol `/2`, and activation mapping `/3` (twenty-one entries total); it asserts exact affected identities, sidecar non-authority and cross-entry independence, and returns unresolved manifests as unexecutable before producer read. The atomic Task-010 assessment path is public only after this resolution gate. QObs is pinned at accepted merge `9240063`; repair/query remain typed unsupported, Quoin intake remains unresolved, and Task-018 owns final record composition. |
| FR-019 | FR-019-AC-1..AC-5 | TC-003 (AC-1..AC-5) | ✅ Implemented — exact activation, false, unavailable and identity-substitution paths execute without business action. |
| FR-020 | FR-020-AC-1..AC-5 | TC-003 (AC-1..AC-5) | ✅ Implemented — registration retains the complete FR-288/compensation-handle/FR-287 key and exact replay adds nothing. |
| FR-021 | FR-021-AC-1..AC-4 | TC-003 (AC-1..AC-4) | ✅ Implemented — strict QObs observation and position-ledger views preserve all three commit-ordering violations distinctly. |
| FR-022 | FR-022-AC-1..AC-5 | TC-003 (AC-1..AC-5) | ✅ Implemented — the initial attempt counts, the authored finite bound admits exactly, and one-over refuses transactionally. |
| FR-023 | FR-023-AC-1..AC-5 | TC-003 (AC-1..AC-5) | ✅ Implemented — attempts, independently established related effects and their closed outcome domains remain separately typed and retained. |
| FR-024 | FR-024-AC-1..AC-6 | TC-003 (AC-1..AC-6); TC-009 (AC-6) | ✅ Implemented — admitted QSL expressions independently report full and partial recovery while retaining the exact operand view; missing/resource outcomes remain non-success, and the callback is compile-fail absent from production. |
| FR-025 | FR-025-AC-1..AC-3 | TC-003, TC-009 (AC-1..AC-3) | ✅ Implemented — capability mutations refuse before assessment and action-capable doubles remain at zero calls. |
| FR-026 | FR-026-AC-1..AC-3 | TC-004 (AC-1, AC-2); none for FR-026-AC-3 | 🚧 Planned — 2 of 3 criteria test-backed. Unbacked: FR-026-AC-3 (Inspection) |
| FR-027 | FR-027-AC-1..AC-5 | TC-004 (AC-1..AC-4); none for FR-027-AC-5 | 🚧 Planned — 4 of 5 criteria test-backed. Unbacked: FR-027-AC-5 (Analysis) |
| FR-028 | FR-028-AC-1..AC-6 | TC-004 (AC-1..AC-6) | 🚧 Planned — 6 of 6 criteria test-backed, 0 unbacked, 0 unexecutable |
| FR-029 | FR-029-AC-1..AC-6 | TC-005 (AC-1..AC-6) | ✅ Complete — ordered total dispositions retain independent supported, refused, unsupported and failed outcomes. |
| FR-030 | FR-030-AC-1..AC-6 | TC-005 (AC-1..AC-6) | ✅ Complete — every exact retained premise axis, backend identity/version and derived correspondence is mutation-tested without fallback repair. |
| FR-031 | FR-031-AC-1..AC-7 | TC-006 (AC-1..AC-7), TC-008 (AC-5) | ✅ Implemented — canonical identity/digest construction uses owner-selected domains and an exact reversible `quire.protocol.runtime-reference/1` carrier for admitted opaque runtime subjects. |
| FR-032 | FR-032-AC-1..AC-7 | TC-006 (AC-1..AC-7) | ✅ Implemented — orthogonal result axes remain typed and are reused from exact owner outputs rather than caller-restated values. |
| FR-033 | FR-033-AC-1..AC-5 | TC-006 (AC-1..AC-5) | ✅ Implemented — strict lineage validation, immutable predecessor/correction edges and identity mutation controls execute before bundle commit. |
| FR-034 | FR-034-AC-1..AC-7 | TC-007 (AC-1..AC-7) | ✅ Complete — exact owner RFC 8785 preimage, every identity-bearing member, target-context equality, source digest and semantic binding order execute at the pinned qverify revision. |
| FR-035 | FR-035-AC-1..AC-7 | TC-007 (AC-1..AC-7) | ✅ Complete — every closed source row and finding axis, adequacy separation, all four dependence relations, owner strict reread and the seven-code refusal boundary execute at the pinned qverify revision. |
| FR-036 | FR-036-AC-1..AC-6 | TC-011 (AC-1..AC-6) | ✅ Implemented — every routed refusal carries the complete typed envelope, exact source/local identity and explicit Protocol usage. |
| FR-037 | FR-037-AC-1..AC-5 | TC-011 (AC-1..AC-5) | ✅ Implemented — identity, equality, retry and location decisions remain independent of display text and locale. |
| FR-038 | FR-038-AC-1..AC-6 | TC-011 (AC-1..AC-6) | ✅ Implemented — closed local causes and exact known/opaque unknown source interiors route without nearest-version interpretation. |
| FR-039 | FR-039-AC-1..AC-6 | TC-011 (AC-1..AC-6); IT-003 (AC-5) | ✅ Implemented — paired and simultaneous causes, order, multiplicity and accounting-fault interiors remain structurally distinct. |
| FR-040 | FR-040-AC-1..AC-4 | TC-011 (AC-1..AC-4); IT-003 (AC-3) | ✅ Implemented — fatal whole-operation paths emit no success and claim-local refusal preserves authorized siblings only. |
| FR-041 | FR-041-AC-1..AC-4 | TC-004 (AC-1..AC-4) | 🚧 Planned — 4 of 4 criteria test-backed |
| FR-042 | FR-042-AC-1..AC-6 | TC-015 (AC-1..AC-6) | ✅ Implemented — all six criteria carry direct traced tests; the current engine reports 3 of 6 criterion rows bound and no FR-042 unbacked or untracked record, so the direct trace inventory and engine rollup remain stated separately |
| FR-301 | FR-301-AC-1..AC-5 | TC-147 (AC-1..AC-5) | 🚧 Planned — refinement is admitted in spec once `agent-ix/quire-protocol#81` lands FR-176/FR-177; TC-147 binds all 5 criteria and no implementation has landed yet. |
| FR-302 | FR-302-AC-1..AC-5 | TC-148 (AC-1..AC-5) | 🚧 Planned — the witness carrier is admitted in spec pending `agent-ix/quire-specification#114`'s `native-run-result/2`; TC-148 binds all 5 criteria and no implementation has landed yet. |

Behavioural totals across these 35 rows: 198 acceptance criteria, of which 193
are backed by a test case, 2 are backed only by an integration scenario
(FR-002-AC-5 and FR-001-AC-5 pass), and 3 are
unbacked — 2 by Inspection and 1 by Analysis.
193 + 2 + 3 = 198.

## Domain Object Coverage

Requirements carrying `object:` specify the consumer-boundary vocabulary. They
are separated from the behavioural table because they define nouns rather than
behaviour.

Sixty-four criteria across the nine. Sixteen of them name no test case and no
integration scenario, and
this table writes `none` for each rather than a test-case id. The remaining
forty-eight name a test case or an integration scenario, and each named test
case declares in its own Traceability section the object criteria it backs, so
the reference resolves in both directions.

| Object Req | Object type | Acceptance Criteria | Test Cases | Status |
| --- | --- | --- | --- | --- |
| FR-010 | domain | FR-010-AC-1..AC-3 | none — all three criteria are Inspection | 🚧 Planned — 0 of 3 criteria test-backed; unbacked: FR-010-AC-1, FR-010-AC-2, FR-010-AC-3 |
| FR-011 | value_object | FR-011-AC-1..AC-7 | TC-014 (AC-2..AC-6); none for FR-011-AC-1, FR-011-AC-7 | 🚧 Planned — 5 of 7 criteria test-backed; unbacked: FR-011-AC-1, FR-011-AC-7 (Inspection) |
| FR-012 | entity | FR-012-AC-1..AC-8 | TC-013 (AC-1..AC-8) | ✅ Implemented — all eight criteria execute in Task-013's ten resolver tests, including exact key-only identity, state independence, two-status absence, interface digest exclusion, containment and deterministic validation order. |
| FR-013 | value_object | FR-013-AC-1..AC-8 | TC-001 (AC-1, AC-2, AC-3, AC-5, AC-6); IT-001 (AC-1..AC-4, AC-6); none for FR-013-AC-7, FR-013-AC-8 | 🚧 Planned — 5 of 8 criteria test-backed; unbacked: FR-013-AC-7 (Inspection), FR-013-AC-8 (Analysis). FR-013-AC-4 now executes in both ordered directions through the explicit `/1` and `/2` entry points and passes with distinct typed causes. |
| FR-014 | value_object | FR-014-AC-1..AC-8 | TC-011 (AC-1, AC-2, AC-4, AC-5, AC-7); IT-003 (AC-5, AC-6); none for FR-014-AC-3, FR-014-AC-8 | 🚧 Planned — 5 of 8 criteria test-backed; unbacked: FR-014-AC-3 (Analysis), FR-014-AC-8 (Inspection); unexecutable: FR-014-AC-6, whose sole backing is IT-003, blocked on the Quoin scenario-record handoff |
| FR-015 | value_object | FR-015-AC-1..AC-5 | TC-014 (AC-2, AC-3, AC-4); TC-001 (AC-2); none for FR-015-AC-1, FR-015-AC-5 | 🚧 Planned — 3 of 5 criteria test-backed; unbacked: FR-015-AC-1, FR-015-AC-5 (Inspection). The implementation exposes only the two exact FR-201 labels selected by FR-015, and Task-006 carries those shared values through final assembly. |
| FR-016 | enumeration | FR-016-AC-1..AC-8 | TC-009 (AC-2..AC-7); TC-011 (AC-5); TC-008 (AC-6); TC-013 (AC-8); IT-001 (AC-2); none for FR-016-AC-1 | 🚧 Planned — 7 of 8 criteria test-backed; unbacked: FR-016-AC-1 (Inspection); 0 unexecutable |
| FR-017 | entity | FR-017-AC-1..AC-8 | TC-016 (AC-1, AC-2, AC-4..AC-8); TC-008, TC-009 (AC-5); TC-010 (AC-7); TC-014 (AC-8); IT-001, IT-002, IT-003 (AC-1, AC-2) | 🚧 Planned — 7 of 8 criteria test-backed; AC-3 is inspection-backed. Task-018 owns final composition after shared outcomes/resources. |
| FR-018 | entity | FR-018-AC-1..AC-9 | TC-017 (AC-1, AC-3, AC-5, AC-6, AC-8, AC-9); none for FR-018-AC-2, FR-018-AC-4, FR-018-AC-7 | ✅ Implemented — all 6 test-designated criteria execute through the consumer-owned `DeliveryIndex`; the three inspection/analysis criteria remain unbacked. The retired pre-delivery corpus is not credited, A's published `MutationManifest` is producer-owned, and Task-013's resolver does not claim this object. |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
| --- | --- | --- | --- |
| NFR-001 | Property and metamorphic testing | TC-008 (AC-1..AC-3) | ✅ Implemented — all 3 criteria are engine-backed. Canonical results retain the complete reproduction tuple; map order, locale-isolated child processes, allocation layout, permitted independent arrival order, authored causal order, producer-integration permutations and E02 projection maps reproduce. Every named result identity and the projection adapter identity independently changes the affected identity or refuses. |
| NFR-002 | Property, mutation, and source-bound inventory inspection | TC-009 (AC-1..AC-4) | ✅ Implemented — one `ResourceAccountant` covers the reconciled admission, linking, assessment, result, resolution, delivery, reproduction and E02 projection sites. Static and authored ceilings admit exact/fresh-exact usage and refuse representable one-short use before work; deadlines report before/after observations and overshoot; inventory/guard mutants and zero-action controls execute. The retired parallel `CountGuard`/`BoundedRun` surface and its dead refusal codes are absent. |
| NFR-003 | Shared Rust language-boundary check, static analysis, architecture inspection, Rust review, Quire coverage, gap analysis | TC-010 (AC-1..AC-6); IT-003 (AC-4, AC-6) | 🚧 Planned — 6 of 6 criteria test-backed, 0 unbacked, 0 unexecutable |
| NFR-004 | Cargo metadata inspection, conditional-compilation inventory, enumerated gate execution over one unchanged head, per-criterion negative controls | TC-012 (AC-1..AC-10) | ✅ Implemented — 10 of 10 criteria are engine-backed by ten executable TC-012 tests. The actual documented commands, manifest and seven conditional sites are checked alongside isolated command/risk, serialization, warning, feature/lane, target and evidence-state mutations. Both real compiler warning-channel controls and the undeclared-feature control refuse as specified. NFR-004-AC-2 remains inspection-method but test-case-backed. |
| NFR-005 | Composition properties and Quire coverage over owed integration lanes | TC-016 (AC-1..AC-6); TC-010 (AC-6); IT-001, IT-002, IT-003 (AC-1..AC-3, AC-6) | 🚧 Planned — 6 of 6 criteria are test-backed. The denominator is all 32 owed scenarios; until Task-018 and the concrete handoffs execute, each metric reports its actual completed count rather than a vacuous percentage. |
| NFR-006 | Cargo manifest, lockfile and rights-record inspection with per-criterion negative controls; delivery-surface tests on B-owned fixtures | TC-013 (AC-6, AC-7); TC-017 (AC-8); none for NFR-006-AC-1..AC-5 | 🚧 In progress — AC-6 and AC-7 execute in Task-013's exact delivery-member controls and AC-8 executes through the consumer `DeliveryIndex`; AC-1..AC-5 remain inspection-backed. `DEPENDENCIES.md` records all seven current git sources, including exact QObs C00, qverify and transitive EA pins. |
| NFR-007 | Public-surface inspection, two-revision surface analysis, and IT-001 cross-version and inherited-axis scenarios | IT-001 (AC-3, AC-4, AC-5, AC-7); none for NFR-007-AC-1, NFR-007-AC-2, NFR-007-AC-6, NFR-007-AC-8 | ✅ Implemented — AC-3 and AC-4 pass both ordered cross-version submissions with distinct typed causes. AC-5 passes the identical historical `/1` corpus before and at the `/2` addition with unchanged admitted identities, per-axis refusals, facade signature/body and `/1` refusal definitions. AC-7's paired current test makes `/1` and `/2` independently refuse all eight inherited `Expected` axes while keeping temporal axes separate. AC-1, AC-2 and AC-8 are inspection evidence; AC-6 is analysis evidence. |
| NFR-008 | Static site counts over the repository's own sources, a rendering-replacement mutation, a captured-stream control, an accessor compilation control, and reproduction comparison | TC-011 (AC-1, AC-2, AC-6); TC-008 (AC-7); none for NFR-008-AC-3, NFR-008-AC-4, NFR-008-AC-5, NFR-008-AC-8 | 🚧 Planned — 4 of 8 criteria test-backed; unbacked: NFR-008-AC-3, NFR-008-AC-4, NFR-008-AC-5, NFR-008-AC-8 (Inspection); 0 unexecutable |

Non-functional totals across these 8 rows: 53 acceptance criteria, of which 36
are backed by a test case, 4 are backed only by an integration scenario
(NFR-007-AC-3, AC-4, AC-5 and AC-7 pass), and 13 are unbacked —
12 by Inspection and 1 by Analysis. 36 + 4 + 13 = 53.

Across all three coverage tables the three per-criterion figures reconcile to
the owed total: 193 + 46 + 36 = 275 test-backed, 2 + 2 + 4 = 8
integration-backed, and 3 + 16 + 13 = 32 unbacked, summing to the 315 authored
acceptance criteria. Of the integration-backed set, 4 pass, 3 remain planned,
and 1 is unexecutable.

## Integration Coverage

| Integration Test | Boundary | Requirements | Status |
| --- | --- | --- | --- |
| IT-001 version 1 | Native compiler `quire.compiled-protocol/1` artifact to protocol library | FR-001, FR-002, FR-009, FR-013, FR-016, FR-017, NFR-002, NFR-005 | ✅ Implemented — SC-01..SC-05 run as 24 default tests from QSL's committed `/1` handoff through explicit admission and local static linking. The positive reader inventory and all numeric/digest/refusal axes pass with no ignores. |
| IT-001 version 2 | Native compiler version 2 artifact to protocol library | FR-001-AC-5, FR-002-AC-5, FR-009, FR-013-AC-4, FR-017, NFR-002, NFR-005, NFR-007 | ✅ Implemented — SC-06 through SC-09 execute under the default local gate, including paired refusal of all eight inherited selection axes, and SC-06a passes the identical 23-case historical `/1` corpus at Protocol `0e5ec292` and `5ce9070e`. The exact `6c8c401` pin supplies both published handoffs. |
| IT-002 | Observation and temporal facts to protocol evaluator/result | FR-003, FR-004, FR-006, FR-009, FR-017, FR-026, FR-041, NFR-002, NFR-005 | 🚧 In progress — Task-010 executes SC-01 through SC-04 through the real QObs/QSL assessment path. Task-021 executes SC-05a..SC-05d and SC-06 with exact QSL/QObs evidence correlation, replay/permutation agreement and seven active composed-budget dimensions. Done Task-020's SC-08 executes the exact twenty-one-contract inventory (including compiled `/2` and activation mapping `/3`), fifteen QObs schema digests, affected-identity and non-authority mutations, and the request-bound resolver-before-read assessment boundary at accepted QObs merge `9240063`. Ten of twelve scenario units pass; SC-07a/SC-07b remain planned under Task-018, and QCI remains downstream only. |
| IT-003 | Protocol result through Engineering Assurance, E02, and Quoin | FR-006, FR-007, FR-009, FR-014, FR-017, FR-039, FR-040, NFR-002, NFR-003, NFR-005 | ⛔ Unexecutable — Engineering Assurance and the exact qverify E02 consumer graph are delivered, but Quoin has no Protocol scenario-record intake, so all owed scenarios score zero rather than passing. |

Owed integration scenarios: 32 across the three integration tests (IT-001 10,
IT-002 12, IT-003 10). All 10 IT-001 and 10 of 12 IT-002 scenario units pass;
IT-002 SC-07a/SC-07b remain planned and all 10 IT-003 scenarios remain
unexecutable. No scenario is reported as failing.

## Test Case Summary

`Traces To` names every requirement whose acceptance criteria cite the test case,
parents and atomic parts alike. A test case whose Traceability section does not
yet name every requirement listed here is a back-edge gap in that test case, not
a coverage claim in this matrix.

| Test ID | Title | Type | Priority | Traces To | Status |
| --- | --- | --- | --- | --- | --- |
| TC-001 | Admit only the selected protocol profile | Property | P0 | FR-001, FR-013, FR-015 | ✅ Implemented — 18 default-running version-2 handoff tests cover exact admission, all owner-declared mutations, independent baseline selection, numeric boundaries, cross-version refusals, inherited-axis parity and every owner resource dimension. They do not claim a local version-2 linked graph; FR-001-AC-5 is closed separately by IT-001/Task-009. |
| TC-002 | Link and discriminate finite protocol structure | Property | P0 | FR-002 | ✅ Implemented — PR #67 (`2a3ef19`) completed the versioned graph/runtime bridge and one-axis identity/content/binding mutation inventory. |
| TC-003 | Assess compensation, retry, commit, and recovery | Property | P0 | FR-003, FR-019, FR-020, FR-021, FR-022, FR-023, FR-024, FR-025 | ✅ Implemented — 8 pure-kernel tests plus 2 graph-owned Engine tests cover QSL evaluation, QObs ordering, finite retry, full/partial recovery, replay, exact missing/resource outcomes, prior-snapshot transactionality and zero action calls. |
| TC-004 | Assess finite global conformance | Property | P0 | FR-004, FR-026, FR-027, FR-028, FR-041 | ✅ Complete — semantic model tests cover legal interleavings, adverse facts and path equivalence; public strict FR-300 `/3` activation frontends cover graph-derived assessment, QObs correlation, compensation synchronization, independent axes and exact result binding. |
| TC-005 | Separate protocol claim dispositions | Property | P0 | FR-005, FR-029, FR-030 | ✅ Complete — semantic cases prove independent ordered dispositions and exact premises; an admitted FR-300 activation fixture proves the public Engine path and compile-fail coverage seals the raw evaluator. |
| TC-006 | Produce and strictly read canonical protocol results | Property | P0 | FR-006, FR-031, FR-032, FR-033 | ✅ Implemented — atomic Engine assessment, sealed stage outputs, derived truth/provenance, private canonical production, strict readback, lineage and runtime-reference mutation coverage execute. |
| TC-007 | Project canonical protocol results to E02 | Property | P0 | FR-007, FR-034, FR-035 | ✅ Complete — four bound Rust symbols pass against exact qverify `293e342`; the manifest, lock and rights record carry the real compatible graph. |
| TC-008 | Reproduce results under equivalent presentations | Property | P0 | NFR-001, FR-016, FR-017, NFR-008 | ✅ Implemented — reproduction inputs, canonical results, producer-integration permutations and E02 projection execute equivalent map/locale/allocation/arrival presentations, authored-order controls and the complete named one-axis identity matrix. |
| TC-009 | Enforce every protocol resource boundary | Property | P0 | NFR-002, FR-003, FR-016, FR-017, FR-025 | ✅ Implemented — the final bidirectional production inventory, inclusive authored/caller bounds, exact/fresh-exact and one-short paths, monotonic deadline observations, inventory/guard mutants, E02 projection sites and zero-action controls execute through the sole supported request accountant. |
| TC-010 | Preserve Rust and shared assurance containment | Static | P0 | NFR-003, FR-009, FR-017 | 🚧 Planned |
| TC-011 | Return typed refusals without text parsing | Property | P0 | FR-008, FR-014, FR-016, FR-036, FR-037, FR-038, FR-039, FR-040, NFR-008 | ✅ Implemented for FR-008/FR-036..FR-040 — foundation/catalog, exact public-route, accounting-fault, decomposition, sibling-retention, text-independence and source-interior tests execute. NFR-008's separately declared static-inspection obligations remain visible in its own row. |
| TC-012 | Run the local verification gate and refuse an undefined lane | Static | P0 | NFR-004 | ✅ Implemented — ten executable tests bind all ten criteria to the exact repository snapshot and thirteen stable defect identities; no test is ignored or environment-gated. |
| TC-013 | Resolve the version-lock manifest | Property | P0 | FR-009, FR-012, FR-016, NFR-006 | ✅ Implemented — eleven Task-013 tests execute deterministic bounded resolution, authentic request-bound capabilities, exact retained identities and exact-member delivery controls; FR-018 is separately assigned to TC-017 and is not pre-credited. |
| TC-014 | Apply the consumer identity discipline | Property | P0 | FR-011, FR-015, FR-017 | ✅ Implemented — B-owned fixtures establish consumer authority and non-substitution without an external dependency. |
| TC-015 | Produce, read and map canonical protocol results | Integration | P0 | FR-042-AC-1..FR-042-AC-6 | ✅ Implemented — 12 executable tests cover canonical production/strict reading, complete typed views, all four independent axes, Boolean/non-value mapping, identity/lineage mutations, schema pins, reproduction and exact/one-short resource limits |
| TC-016 | Compose complete scenario evidence without storing it | Property | P0 | FR-017, NFR-005 | 🚧 Planned — final outcome/resource types, deterministic partial resolution, per-axis distinctness and transient-value containment |
| TC-017 | Validate the consumer-owned delivery index | Property | P0 | FR-018, NFR-006 | ✅ Implemented — five executable index tests cover exact identity/state, bytes, typed absence, temporal/input/mutation/limit controls, bounded retention and version-lock non-authority. |
| TC-147 | Admit protocol refinement dispatch | Property | P0 | FR-301 | 🚧 Planned — binds all 5 FR-301 criteria; awaits `agent-ix/quire-protocol#81` landing FR-176/FR-177. |
| TC-148 | Carry a trace-position witness on the protocol result wire | Property | P0 | FR-302 | 🚧 Planned — binds all 5 FR-302 criteria; awaits `agent-ix/quire-specification#114`'s `native-run-result/2`. |

## Test Execution Summary

Row counts, recomputed from the tree. Every matrix row above is counted exactly
once, and `Blocked` means unexecutable — neither passing nor failing.

Engine-measured binding at the current implementation checkpoint is 175 of 354
rows (49%) — see **Engine-bound** above. The table below counts matrix rows and
their planned state; it does not measure binding.

| Category | Total | Passed | Failed | Blocked | Planned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Functional requirement rows | 35 | 2 | 0 | 0 | 33 |
| Domain object requirement rows | 9 | 1 | 0 | 0 | 8 |
| Non-functional requirement rows | 8 | 3 | 0 | 0 | 5 |
| Integration rows | 4 | 2 | 0 | 1 | 1 |
| Test case rows | 19 | 5 | 0 | 0 | 14 |
| Overall | 75 | 13 | 0 | 1 | 61 |

The Integration count is four because IT-001 carries a separate version 1 and version 2
row; the two contracts may not substitute for each other, so one row cannot
report both. Both IT-001 rows pass. IT-002 remains planned and IT-003 remains
unexecutable.

### Owed acceptance criteria

`NFR-005` denominates on what each lane **owes**, not on what it executed, so a
fully blocked integration cannot score 100%. These totals follow that rule: the
denominator is every authored acceptance criterion, and an unexecutable
criterion is a zero inside it rather than a row removed from it.

| Category | Total | Passed | Failed | Blocked | Planned |
| --- | ---: | ---: | ---: | ---: | ---: |
| Criteria backed by a test case | 275 | 35 | 0 | 0 | 240 |
| Criteria backed only by an integration scenario | 8 | 4 | 0 | 1 | 3 |
| Criteria with no test case and no integration scenario | 32 | 0 | 0 | 0 | 32 |
| Overall owed criteria | 315 | 39 | 0 | 1 | 275 |

Task-013 adds eleven completed criteria to this table: all eight FR-012 object
criteria, FR-016-AC-8, and NFR-006-AC-6/AC-7. FR-009's nine resolver controls
execute, but its aggregate criteria remain in progress until the concrete
producer integrations they also name run; they are not double-counted as
complete here. Task-019 adds the six test-designated FR-018 criteria and
NFR-006-AC-8 through TC-017; its three inspection/analysis criteria remain
unbacked.

The four passing integration-only criteria are FR-002-AC-5, FR-013-AC-4,
NFR-007-AC-3 and NFR-007-AC-4. FR-001-AC-5, NFR-007-AC-5 and NFR-007-AC-7 remain
planned because their complete version 1 regression or inherited-axis comparison is
not established by Task-012. The one unexecutable criterion is FR-014-AC-6,
waiting on IT-003's Quoin scenario-record handoff; Engineering Assurance and
the exact E02 consumer graph are delivered.

The 32 unbacked criteria are FR-008-AC-9, FR-010-AC-1, FR-010-AC-2,
FR-010-AC-3, FR-011-AC-1, FR-011-AC-7, FR-013-AC-7, FR-013-AC-8,
FR-014-AC-3, FR-014-AC-8, FR-015-AC-1, FR-015-AC-5, FR-016-AC-1,
FR-017-AC-3, FR-018-AC-2, FR-018-AC-4, FR-018-AC-7, FR-026-AC-3,
FR-027-AC-5, NFR-006-AC-1, NFR-006-AC-2, NFR-006-AC-3, NFR-006-AC-4,
NFR-006-AC-5, NFR-007-AC-1, NFR-007-AC-2, NFR-007-AC-6, NFR-007-AC-8,
NFR-008-AC-3, NFR-008-AC-4, NFR-008-AC-5 and NFR-008-AC-8. Each verifies by
Inspection or Analysis and none is claimed as test coverage. TC-017 is the final
test-case id and owns DeliveryIndex evidence formerly over-credited to TC-013.
