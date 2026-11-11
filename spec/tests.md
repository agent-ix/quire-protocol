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
  - { target: ix://agent-ix/quire-protocol/FR-043, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-044, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-045, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-046, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-047, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-050, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-051, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-052, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-053, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-054, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-055, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-056, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-057, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-058, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-059, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-100, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-101, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-150, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-151, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-152, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-153, type: references }
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

The repository carries 65 functional requirements, 8 non-functional
requirements, 3 integration tests and 35 test cases, and 430 acceptance criteria
across the 65 FRs and the 8 NFRs. The functional set divides into 9 aggregate or
behavioural parents (FR-001..FR-009), 9 object requirements (FR-010..FR-018),
24 atomic behavioural parts (FR-019..FR-042) that the parents aggregate
through `contains`, 2 independent behavioural requirements (FR-301, FR-302)
that extend FR-005 and FR-006's claim dispatch and result-wire obligations
rather than being decomposed from them, 5 checked-package requirements
(FR-043..FR-047), 10 extension requirements for dynamic roles, queues, joins and
loops (FR-050..FR-059, ADR-010), and 2 core-protocol delta requirements
(FR-100, FR-101, ADR-010). FR-003 through FR-008 now state their
obligations once each through those parts; the parent row and the part rows are
therefore two views of one obligation and must not be added together as
separate work.

The 403 counts rows of a `## Acceptance Criteria` table on an FR or an NFR, and
nothing else. `quire properties` reports 423 criteria over the same tree: the
extra 20 are the `StR-0NN-VC-n` validation criteria of StR-001 through StR-005,
which are stakeholder validation rather than FR/NFR acceptance and which this
matrix carries no table for. NFR-005's five metrics are outside that figure;
its six acceptance criteria are included.

### Stage 1 of the checked-package repoint (QSL-1549)

QSL main removed the `wire::Package` admission, the compiled-protocol readers,
`native_temporal`, `state` and `ByteDigest`. The first repoint stage therefore
deleted every consumer of them and their tests: the link layer, the version-2
model, the assessment, the conformance kernel, the claim dispatch, the result
record and its reader, the E02 projection, the producer-contract registry, the
inventory map and the compensation graph, with TC-002, TC-004, TC-005, TC-006
and the IT-001/IT-002 procedures. The status of the FR rows 001 to 042 below
describes the implementation that stage retired; their successors are FR-044 to
FR-047 (stages 2 and 3, now landed in part), and no row of the retired implementation is claimed
as passing by this stage.

### Row status symbols

- ✅ Implemented or Complete: the row's tests are authored and pass.
- 🚧 Planned or In progress: the row is specified; its tests are not all authored
  or not all passing.
- ⛔ Blocked: the row is specified but cannot be implemented or tested until a
  named dependency owned elsewhere is delivered; the row names the dependency.
- ⛔ Retired: the row's tests were deleted with the implementation they
  exercised; the row claims nothing and names its successor where one exists.

### What each status word in this matrix means

- **Living-test** — the criterion carries a `#[trace(...)]` attribute on a test in
  the current tree. 136 of 344 criteria. This is the only claim
  of passing this matrix makes, and it is read from the tree, not from a
  `Verification` cell.
- **Designated** — the criterion's `Verification` cell names a test case or an
  integration scenario. A designation is not a living test: QSL-1549 stage 1
  deleted the tests behind TC-001, TC-002, TC-004, TC-005, TC-006, TC-007,
  TC-014 and TC-015 and the IT-001 and IT-002 procedures, so criteria designated
  to them are retired or planned until their successors land.
- **Unbacked** — the cell names neither; the criterion verifies by Inspection or
  Analysis only and is not covered by a test.
- **Unexecutable** — a backing exists but cannot run (IT-003, FR-014-AC-6). Under
  `FR-009-AC-3` such a scenario is counted as zero, never as passing and never
  omitted from a denominator.

Each row's Status cell gives the per-criterion count and is the figure to read.
The `quire coverage` engine census and the engine-bound row count are not
restated here: earlier revisions of this file quoted figures measured over the
tree before stage 1, and a figure for the current tree is read by running
`quire coverage --scope .`.

### Where the retired behaviour goes

The link layer, the compensation assessment over the checked package and the
temporal evidence route return as FR-044, FR-045 and FR-046. Result, claim
dispatch, the E02 projection and the producer-contract registry have no
successor requirement yet; their rows stay retired until one is specified.

## Functional Requirement Coverage

Parent and atomic rows are both listed. FR-010..FR-018 carry `object:` and are
in Domain Object Coverage instead.

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- |
| FR-001 | FR-001-AC-1, FR-001-AC-2, FR-001-AC-3, FR-001-AC-4, FR-001-AC-5 | no living test for AC-1..AC-5 | ⛔ Retired — every test of this row's 5 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised (successor: FR-043). |
| FR-002 | FR-002-AC-1, FR-002-AC-2, FR-002-AC-3, FR-002-AC-4, FR-002-AC-5 | no living test for AC-1..AC-5 | ⛔ Retired — every test of this row's 5 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised (successor: FR-044). |
| FR-003 | FR-003-AC-1, FR-003-AC-2, FR-003-AC-3, FR-003-AC-4, FR-003-AC-5, FR-003-AC-6, FR-003-AC-7 | TC-003 (AC-1..AC-7); TC-009 (AC-7) | ✅ Implemented — TC-003's eight tests exercise the compensation kernels (activation, registration, attempt and effect accounting, commit ordering, retry bound, recovery) over QObs views with zero action calls. Aggregate view of FR-019..FR-025; evaluation through the checked package returns with FR-045. |
| FR-004 | FR-004-AC-1, FR-004-AC-2, FR-004-AC-3, FR-004-AC-4 | no living test for AC-1..AC-4 | ⛔ Retired — every test of this row's 4 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised (successor: FR-046). |
| FR-005 | FR-005-AC-1, FR-005-AC-2, FR-005-AC-3, FR-005-AC-4, FR-005-AC-5 | no living test for AC-1..AC-5 | ⛔ Retired — every test of this row's 5 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-006 | FR-006-AC-1..AC-7, FR-006-AC-9 | no living test for AC-1..AC-7, AC-9 | ⛔ Retired — every test of this row's 8 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-007 | FR-007-AC-1, FR-007-AC-2, FR-007-AC-3, FR-007-AC-4, FR-007-AC-5, FR-007-AC-6 | no living test for AC-1..AC-6 | ⛔ Retired — every test of this row's 6 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-008 | FR-008-AC-1, FR-008-AC-2, FR-008-AC-3, FR-008-AC-4, FR-008-AC-5, FR-008-AC-6, FR-008-AC-7, FR-008-AC-8, FR-008-AC-9 | TC-011 (AC-1..AC-3, AC-7); no living test for AC-4..AC-6, AC-8..AC-9 | 🚧 Planned — 4 of 9 criteria have a living traced test; 4 designated to tests deleted in QSL-1549 stage 1; 1 verify by Inspection or Analysis only. |
| FR-009 | FR-009-AC-1..FR-009-AC-9 | TC-013 (AC-1..AC-9) | 🚧 In progress — TC-013's twelve version-lock tests execute the resolver controls of all nine criteria with deterministic bounded request-bound capabilities. The aggregate criteria stay in progress until concrete producer integrations run; the IT-001 and IT-002 procedures that named them are retired. |
| FR-019 | FR-019-AC-1..AC-5 | TC-003 (AC-1..AC-5) | ✅ Implemented — exact activation, false, unavailable and identity-substitution paths execute without business action. |
| FR-020 | FR-020-AC-1..AC-5 | TC-003 (AC-1..AC-5) | ✅ Implemented — registration retains the complete FR-288/compensation-handle/FR-287 key and exact replay adds nothing. |
| FR-021 | FR-021-AC-1..AC-4 | TC-003 (AC-1..AC-4) | ✅ Implemented — strict QObs observation and position-ledger views preserve all three commit-ordering violations distinctly. |
| FR-022 | FR-022-AC-1..AC-5 | TC-003 (AC-1..AC-5) | ✅ Implemented — the initial attempt counts, the authored finite bound admits exactly, and one-over refuses transactionally. |
| FR-023 | FR-023-AC-1..AC-5 | TC-003 (AC-1..AC-5) | ✅ Implemented — attempts, independently established related effects and their closed outcome domains remain separately typed and retained. |
| FR-024 | FR-024-AC-1..AC-6 | no living test for AC-1..AC-6 | ⛔ Retired — every test of this row's 6 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised (successor: FR-045). |
| FR-025 | FR-025-AC-1..AC-3 | TC-003 (AC-1..AC-3); TC-009 (AC-1..AC-3) | ✅ Implemented — capability mutations refuse before assessment and action-capable doubles remain at zero calls. |
| FR-026 | FR-026-AC-1..AC-3 | no living test for AC-1..AC-3 | ⛔ Retired — every test of this row's 3 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-027 | FR-027-AC-1..AC-5 | no living test for AC-1..AC-5 | ⛔ Retired — every test of this row's 5 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-028 | FR-028-AC-1..AC-6 | no living test for AC-1..AC-6 | ⛔ Retired — every test of this row's 6 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-029 | FR-029-AC-1..AC-6 | no living test for AC-1..AC-6 | ⛔ Retired — every test of this row's 6 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-030 | FR-030-AC-1..AC-6 | no living test for AC-1..AC-6 | ⛔ Retired — every test of this row's 6 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-031 | FR-031-AC-1, FR-031-AC-4..AC-7 | no living test for AC-1, AC-4..AC-7 | ⛔ Retired — every test of this row's 5 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-032 | FR-032-AC-1..AC-7 | no living test for AC-1..AC-7 | ⛔ Retired — every test of this row's 7 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-033 | FR-033-AC-1..AC-5 | no living test for AC-1..AC-5 | ⛔ Retired — every test of this row's 5 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-034 | FR-034-AC-1..AC-7 | no living test for AC-1..AC-7 | ⛔ Retired — every test of this row's 7 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-035 | FR-035-AC-1..AC-7 | no living test for AC-1..AC-7 | ⛔ Retired — every test of this row's 7 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-036 | FR-036-AC-1..AC-6 | TC-011 (AC-1..AC-2, AC-4..AC-5); no living test for AC-3, AC-6 | 🚧 Planned — 4 of 6 criteria have a living traced test; 2 designated to tests deleted in QSL-1549 stage 1. |
| FR-037 | FR-037-AC-1..AC-5 | TC-011 (AC-1, AC-3, AC-5); no living test for AC-2, AC-4 | 🚧 Planned — 3 of 5 criteria have a living traced test; 2 designated to tests deleted in QSL-1549 stage 1. |
| FR-038 | FR-038-AC-1..AC-6 | TC-011 (AC-1..AC-4); no living test for AC-5..AC-6 | 🚧 Planned — 4 of 6 criteria have a living traced test; 2 designated to tests deleted in QSL-1549 stage 1. |
| FR-039 | FR-039-AC-1..AC-6 | TC-011 (AC-1..AC-3); no living test for AC-4..AC-6 | 🚧 Planned — 3 of 6 criteria have a living traced test; 3 designated to tests deleted in QSL-1549 stage 1. |
| FR-040 | FR-040-AC-1..AC-4 | TC-011 (AC-1..AC-2); no living test for AC-3..AC-4 | 🚧 Planned — 2 of 4 criteria have a living traced test; 2 designated to tests deleted in QSL-1549 stage 1. |
| FR-041 | FR-041-AC-1..AC-4 | no living test for AC-1..AC-4 | ⛔ Retired — every test of this row's 4 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-042 | FR-042-AC-1..AC-6 | no living test for AC-1..AC-6 | ⛔ Retired — every test of this row's 6 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-301 | FR-301-AC-1..AC-6 | TC-147 (AC-1..AC-6) | 🚧 Planned — refinement is dispatched to QSL's refinement backend (QSL implements FR-176/FR-177, ADR-010); TC-147 binds all 6 criteria and no dispatch implementation has landed yet. |
| FR-302 | FR-302-AC-1..AC-5 | TC-148 (AC-1..AC-5) | 🚧 Planned — the witness carrier is admitted in spec pending `agent-ix/quire-specification#114`'s `native-run-result/2`; TC-148 binds all 5 criteria and no implementation has landed yet. |
| FR-043 | FR-043-AC-1..AC-8 | TC-149 (AC-1..AC-8) | ✅ Implemented — TC-149's eight tests admit the three packages of recipe R, replay the 30-case standard mutation corpus, and carry the reader's, the selection's and the identity and equality files' refusals unchanged. |
| FR-044 | FR-044-AC-1..AC-7 | TC-150 (AC-1..AC-7) | ✅ Implemented — all 7 criteria have living traced tests. AC-1 compares lists and edges with `protocol_view`/`causal_relation`; AC-2 forbids local edge construction; AC-3 checks the consumer-owned refusals with their node ids and loci and that the linker builds no owner refusal; AC-4 checks absent, carrier and non-protocol refusals; AC-5 checks the checked-package signature and retired types; AC-6 checks the linker names no window view and links without a window package member; AC-7 checks, over recipe R, distinct concurrent branch identities and join edges, unordered branches that all reach the join, no new edge from reordered clock-selection input, and the retained selected clock binding. |
| FR-045 | FR-045-AC-1..AC-8 | TC-151 (AC-1, AC-3, AC-4, AC-6, AC-8); no living test for AC-2, AC-5, AC-7 | 🚧 In progress — 5 of 8 criteria have a living traced test over `BarePackage`'s slot call (QSL FR-531, FR-533): AC-1 covers the guard and retry rows, the unavailable binder, the wrongly typed value and the meter one below usage; AC-3 the unavailable retry binders, the value-asserted mappings of every non-Boolean owner outcome and the absence of Boolean coercion; AC-4 the `attempts maximum` bound of 3 admitting three attempts and refusing the fourth; AC-6 the retired names; AC-8 the per-binder charge log. Owed: R's `recovery.predicate` slots cannot be evaluated because QSL's bare projection faults on the `sum(receipt in ...)` binder (`reader-expression-binder-is-admitted`), so AC-1's recovery rows, AC-2's restated recovery assertions and AC-3's recovery supplies wait on that QSL fix; an unsupported-construct row has no input in R; AC-5 has no denied-syscall harness; AC-7 ends because AC-8's gate is met. The retry bound is read from the node's `attempts` argument directly because QSL's `protocol_view` has no accessor for it. |
| FR-046 | FR-046-AC-1..AC-7 | TC-152 (AC-1, AC-2, AC-3, AC-4, AC-6); no living test for AC-5, AC-7 | 🚧 In progress — 5 of 7 criteria have a living traced test over `EvidencePackage::from_package`: AC-1 the record equals QSL's result for R's two named clauses and changes with a valuation, AC-2 and AC-3 no retired contract string or `native_temporal` in source, AC-4 six reachable causes of QSL FR-516's table come back by name with nothing charged, AC-6 a missing or withdrawn anchor settles as a missing non-value. Owed: AC-4 does not cover the causes a request over R cannot reach; AC-5 has no input, since the reader refuses the name rows that would give the one bytes-route construction refusal (`AmbiguousName`); AC-7 (the retained `temporal::` surface and `Position.instance_valuations`) has no test; AC-6's IT-002 scenarios are not redone (see FR-047-AC-3). The request is built by the caller: no mapping from observation evidence to `EvidenceRequest` is specified. |
| FR-047 | FR-047-AC-1..AC-4 | TC-153 (AC-1, AC-2, AC-4); no living test for AC-3 | 🚧 In progress — 3 of 4 criteria have a living traced test: no `quire_spec_language::number`, no `handoff-writer` feature or retired format identity in source, and no checksum, seal or `ArtifactRef` (the planner ruled for removal). AC-3 is owed: IT-001 and IT-002 are still the retired procedures, and their scenarios have no counterpart table against recipe R yet. |
| FR-050 | FR-050-AC-1..AC-4 | TC-050 (AC-1..AC-4) | 🚧 Planned — spec only (ADR-010); binds all 4 criteria; awaits the implementation. |
| FR-051 | FR-051-AC-1..AC-3 | TC-050 (AC-1..AC-3) | 🚧 Planned — spec only (ADR-010); binds all 3 criteria; awaits the implementation. |
| FR-052 | FR-052-AC-1..AC-3 | TC-050 (AC-1..AC-3) | 🚧 Planned — spec only (ADR-010); binds all 3 criteria; awaits the implementation. |
| FR-053 | FR-053-AC-1..AC-8 | TC-051 (AC-1..AC-8) | 🚧 Planned — spec only (ADR-010); binds all 8 criteria; awaits the implementation. |
| FR-054 | FR-054-AC-1..AC-7 | TC-052 (AC-1..AC-7) | 🚧 Planned — spec only (ADR-010); binds all 7 criteria; awaits the implementation. |
| FR-055 | FR-055-AC-1..AC-6 | TC-053 (AC-1..AC-6) | 🚧 Planned — spec only (ADR-010); binds all 6 criteria; awaits the implementation. |
| FR-056 | FR-056-AC-1..AC-4 | TC-054 (AC-1..AC-4) | 🚧 Planned — spec only (ADR-010); binds all 4 criteria; awaits the implementation. |
| FR-057 | FR-057-AC-1..AC-4 | TC-055 (AC-1..AC-4) | 🚧 Planned — spec only (ADR-010); binds all 4 criteria; awaits the implementation. |
| FR-058 | FR-058-AC-1..AC-3 | TC-055 (AC-1..AC-3) | 🚧 Planned — spec only (ADR-010); binds all 3 criteria; awaits the implementation. |
| FR-059 | FR-059-AC-1..AC-4 | TC-054 (AC-1..AC-4) | 🚧 Planned — spec only (ADR-010); binds all 4 criteria; awaits the implementation. |
| FR-100 | FR-100-AC-1..AC-6 | TC-154 (AC-1..AC-6) | 🚧 Planned — spec only (ADR-010); binds all 6 criteria; closes V1-PROT-019 timeout branches; awaits the implementation. |
| FR-101 | FR-101-AC-1..AC-5 | TC-154 (AC-1..AC-5) | 🚧 Planned — spec only (ADR-010); binds all 5 criteria; closes V1-PROT-019 captures; awaits the implementation. |
| FR-150 | FR-150-AC-1..AC-9 | TC-160 (AC-1..AC-9) | 🚧 Planned — spec only (ADR-010); binds all 9 criteria; awaits the implementation and, for a live result, a QSL backend. |
| FR-151 | FR-151-AC-1..AC-6 | TC-161 (AC-1..AC-6) | 🚧 Planned — spec only (ADR-010); binds all 6 criteria; awaits the implementation and, for a live result, a QSL backend. |
| FR-152 | FR-152-AC-1..AC-5 | TC-162 (AC-1..AC-5) | 🚧 Planned — spec only (ADR-010); binds all 5 criteria; awaits the implementation and, for a live result, a QSL backend. |
| FR-153 | FR-153-AC-1..AC-7 | TC-163 (AC-1..AC-7) | 🚧 Planned — spec only (ADR-010); binds all 7 criteria; awaits the implementation and, for a live result, a QSL backend. |

Behavioural totals across these 56 rows: 315 acceptance criteria. 88 carry a living traced test, 106 belong to retired rows (their tests were deleted in QSL-1549 stage 1), 0 are blocked and 121 are planned or verify by Inspection or Analysis only. 88 + 106 + 0 + 121 = 315.

## Domain Object Coverage

Requirements carrying `object:` specify the consumer-boundary vocabulary. They
are separated from the behavioural table because they define nouns rather than
behaviour.

The nine object requirements hold 62 acceptance criteria. 28 carry a living traced test, 19 belong to retired rows and 15 are planned or verify by Inspection or Analysis only.

| Object Req | Object type | Acceptance Criteria | Test Cases | Status |
| --- | --- | --- | --- | --- |
| FR-010 | domain | FR-010-AC-1..AC-3 | no living test for AC-1..AC-3 | 🚧 Planned — 0 of 3 criteria have a living traced test; 3 verify by Inspection or Analysis only. |
| FR-011 | value_object | FR-011-AC-1..AC-7 | TC-008 (AC-2, AC-6); no living test for AC-1, AC-3..AC-5, AC-7 | 🚧 Planned — 2 of 7 criteria have a living traced test; 3 designated to tests deleted in QSL-1549 stage 1; 2 verify by Inspection or Analysis only. |
| FR-012 | entity | FR-012-AC-1..AC-8 | TC-013 (AC-1..AC-8) | ✅ Implemented — all eight criteria execute in Task-013's ten resolver tests, including exact key-only identity, state independence, two-status absence, interface digest exclusion, containment and deterministic validation order. |
| FR-013 | value_object | FR-013-AC-1..AC-8 | no living test for AC-1..AC-8 | ⛔ Retired — every test of this row's 8 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-014 | value_object | FR-014-AC-1..AC-8 | TC-011 (AC-1..AC-2, AC-4..AC-5); no living test for AC-3, AC-6..AC-8 | 🚧 Planned — 4 of 8 criteria have a living traced test; 1 designated to tests deleted in QSL-1549 stage 1; 2 verify by Inspection or Analysis only; 1 depend on a planned or unexecutable test. |
| FR-015 | value_object | FR-015-AC-1, FR-015-AC-2, FR-015-AC-5 | no living test for AC-1..AC-2, AC-5 | ⛔ Retired — every test of this row's 3 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-016 | enumeration | FR-016-AC-1..AC-8 | TC-008 (AC-6); TC-009 (AC-2..AC-7); TC-011 (AC-5); TC-013 (AC-8); traced test (AC-1) | ✅ Implemented — all 8 criteria carry a living traced test across TC-008, TC-009, TC-011 and TC-013 (reproduction record, bounds, refusal catalog and version-lock tests); FR-016-AC-1 is an Inspection criterion also carried by a traced test. |
| FR-017 | entity | FR-017-AC-1..AC-8 | no living test for AC-1..AC-8 | ⛔ Retired — every test of this row's 8 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| FR-018 | entity | FR-018-AC-1..AC-9 | TC-017 (AC-1, AC-3, AC-5..AC-6, AC-8..AC-9); no living test for AC-2, AC-4, AC-7 | ✅ Implemented — the six test-designated criteria execute through the consumer-owned `DeliveryIndex`; the three inspection/analysis criteria remain unbacked. Task-013's resolver does not claim this object. |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
| --- | --- | --- | --- |
| NFR-001 | Property and metamorphic testing | TC-008 (AC-1..AC-3) | ✅ Implemented — TC-008's thirteen tests bind all 3 criteria for the reproduction record: seed, toolchain and corpus identities are typed fields, absence refuses with its own code and never becomes zero, display text is not identity, and equal inputs give equal bytes. Result-level reproduction returns with the result successor. |
| NFR-002 | Property, mutation, and source-bound inventory inspection | TC-009 (AC-1..AC-4) | ✅ Implemented — one `ResourceAccountant` covers the reconciled production inventory of the delivery-index, version-lock, reproduction and compensation work sites. Static and authored ceilings admit exact usage and refuse one-short use before work; deadlines report before and after observations. The dimensions whose sites left with the deleted modules are named as awaiting their successor stage. |
| NFR-003 | Static analysis, architecture inspection, Rust review, Quire coverage, gap analysis | no living test for AC-1..AC-6 | 🚧 Planned — 0 of 6 criteria have a living traced test; 6 depend on a planned or unexecutable test. |
| NFR-004 | Cargo metadata inspection, conditional-compilation inventory, enumerated gate execution over one unchanged head, per-criterion negative controls | TC-012 (AC-1..AC-10) | ✅ Implemented — all 10 criteria are bound by TC-012's eleven executable tests over the documented four-command gate, the manifest, the conditional-compilation inventory and isolated command, risk, serialization, warning, feature, target and evidence-state mutations; the crate declares no feature and runs one lane. |
| NFR-005 | Composition properties and Quire coverage over owed integration lanes | no living test for AC-1..AC-6 | 🚧 Planned — 0 of 6 criteria have a living traced test; 6 depend on a planned or unexecutable test. |
| NFR-006 | Cargo manifest, lockfile and rights-record inspection with per-criterion negative controls; delivery-surface tests on B-owned fixtures | TC-013 (AC-6..AC-7); no living test for AC-1..AC-5, AC-8 | 🚧 Planned — 2 of 8 criteria have a living traced test; 1 designated to tests deleted in QSL-1549 stage 1; 5 verify by Inspection or Analysis only. |
| NFR-007 | Public-surface inspection, two-revision surface analysis, and IT-001 cross-version and inherited-axis scenarios | no living test for AC-1..AC-8 | ⛔ Retired — every test of this row's 8 criteria was deleted in QSL-1549 stage 1 with the implementation it exercised. |
| NFR-008 | Static site counts over the repository's own sources, a rendering-replacement mutation, a captured-stream control, an accessor compilation control, and reproduction comparison | TC-011 (AC-6); no living test for AC-1..AC-5, AC-7..AC-8 | 🚧 Planned — 1 of 8 criteria have a living traced test; 3 designated to tests deleted in QSL-1549 stage 1; 4 verify by Inspection or Analysis only. |

Non-functional totals across these 8 rows: 53 acceptance criteria. 20 carry a living traced test, 8 belong to retired rows (their tests were deleted in QSL-1549 stage 1), 0 are blocked and 25 are planned or verify by Inspection or Analysis only. 20 + 8 + 0 + 25 = 53.

## Integration Coverage

| Integration Test | Boundary | Requirements | Status |
| --- | --- | --- | --- |
| IT-001 version 1 | Native compiler `quire.compiled-protocol/1` artifact to protocol library | FR-001, FR-002, FR-009, FR-013, FR-016, FR-017, NFR-002, NFR-005 | ⛔ Retired — the IT-001/IT-002 procedures were deleted in QSL-1549 stage 1; FR-047-AC-3 owns their replacement on recipe R. |
| IT-001 version 2 | Native compiler version 2 artifact to protocol library | FR-001-AC-5, FR-002-AC-5, FR-009, FR-013-AC-4, FR-017, NFR-002, NFR-005, NFR-007 | ⛔ Retired — the IT-001/IT-002 procedures were deleted in QSL-1549 stage 1; FR-047-AC-3 owns their replacement on recipe R. |
| IT-002 | Observation and temporal facts to protocol evaluator/result | FR-003, FR-004, FR-006, FR-009, FR-017, FR-026, FR-041, NFR-002, NFR-005 | ⛔ Retired — the IT-001/IT-002 procedures were deleted in QSL-1549 stage 1; FR-047-AC-3 owns their replacement on recipe R. |
| IT-003 | Protocol result through Engineering Assurance, E02, and Quoin | FR-006, FR-007, FR-009, FR-014, FR-017, FR-039, FR-040, NFR-002, NFR-003, NFR-005 | ⛔ Unexecutable — Engineering Assurance and the exact qverify E02 consumer graph are delivered, but Quoin has no Protocol scenario-record intake, so all owed scenarios score zero rather than passing. |

Owed integration scenarios: 32 across the three integration tests (IT-001 10,
IT-002 12, IT-003 10). The IT-001 and IT-002 procedures and their tests were
deleted in QSL-1549 stage 1, so none of their 22 scenarios passes in the current
tree; FR-047-AC-3 owns their replacement on recipe R. All 10 IT-003 scenarios
remain unexecutable. No scenario is reported as failing.

## Test Case Summary

`Traces To` names every requirement whose acceptance criteria cite the test case,
parents and atomic parts alike. A test case whose Traceability section does not
yet name every requirement listed here is a back-edge gap in that test case, not
a coverage claim in this matrix.

| Test ID | Title | Type | Priority | Traces To | Status |
| --- | --- | --- | --- | --- | --- |
| TC-001 | Admit only the selected protocol profile | Property | P0 | FR-001, FR-013, FR-015 | ⛔ Retired — its tests were deleted in QSL-1549 stage 1 with the implementation they exercised. |
| TC-002 | Link and discriminate finite protocol structure | Property | P0 | FR-002 | ⛔ Retired — its tests were deleted in QSL-1549 stage 1 with the implementation they exercised. |
| TC-003 | Assess compensation, retry, commit, and recovery | Property | P0 | FR-003, FR-019, FR-020, FR-021, FR-022, FR-023, FR-024, FR-025 | ✅ Implemented — eight tests cover the compensation kernels: activation, registration, replay, commit ordering, finite retry, full and partial recovery, exact missing and resource outcomes, and zero action calls. Evaluation through the checked package returns with TC-151. |
| TC-004 | Assess finite global conformance | Property | P0 | FR-004, FR-026, FR-027, FR-028, FR-041 | ⛔ Retired — its tests were deleted in QSL-1549 stage 1 with the implementation they exercised. |
| TC-005 | Separate protocol claim dispositions | Property | P0 | FR-005, FR-029, FR-030 | ⛔ Retired — its tests were deleted in QSL-1549 stage 1 with the implementation they exercised. |
| TC-006 | Produce and strictly read canonical protocol results | Property | P0 | FR-006, FR-031, FR-032, FR-033 | ⛔ Retired — its tests were deleted in QSL-1549 stage 1 with the implementation they exercised. |
| TC-007 | Project canonical protocol results to E02 | Property | P0 | FR-007, FR-034, FR-035 | ⛔ Retired — its tests were deleted in QSL-1549 stage 1 with the implementation they exercised. |
| TC-008 | Reproduce results under equivalent presentations | Property | P0 | NFR-001, FR-016, FR-017, NFR-008 | ✅ Implemented — thirteen tests cover the reproduction record: typed seed, toolchain and corpus identities, distinct absent and empty refusals, display text excluded from identity, and deterministic bytes. |
| TC-009 | Enforce every protocol resource boundary | Property | P0 | NFR-002, FR-003, FR-016, FR-017, FR-025 | ✅ Implemented — thirteen tests cover the bidirectional production inventory, inclusive authored and caller bounds, exact and one-short paths, monotonic deadline observations, inventory and guard mutants and zero-action controls through the request accountant; dimensions whose sites were deleted are listed as awaiting their successor. |
| TC-010 | Preserve Rust and shared assurance containment | Static | P0 | NFR-003, FR-009, FR-017 | 🚧 Planned |
| TC-011 | Return typed refusals without text parsing | Property | P0 | FR-008, FR-014, FR-016, FR-036, FR-037, FR-038, FR-039, FR-040, NFR-008 | ✅ Implemented for FR-008 and FR-036..FR-040 (the criteria with living tests) — the catalog integration tests and the refusal unit tests execute; the criteria of those rows without a living test are counted in their own rows, and NFR-008's static-inspection obligations remain in its row. |
| TC-012 | Run the local verification gate and refuse an undefined lane | Static | P0 | NFR-004 | ✅ Implemented — eleven executable tests bind all ten criteria to the repository snapshot and thirteen stable defect identities over the four-command gate; no test is ignored or environment-gated. |
| TC-013 | Resolve the version-lock manifest | Property | P0 | FR-009, FR-012, FR-016, NFR-006 | ✅ Implemented — twelve version-lock tests execute deterministic bounded resolution, request-bound capabilities, exact retained identities and exact-member delivery controls; FR-018 is separately assigned to TC-017. |
| TC-014 | Apply the consumer identity discipline | Property | P0 | FR-011, FR-015, FR-017 | ⛔ Retired — its tests were deleted in QSL-1549 stage 1 with the implementation they exercised. |
| TC-015 | Produce, read and map canonical protocol results | Integration | P0 | FR-042-AC-1..FR-042-AC-6 | ⛔ Retired — its tests were deleted in QSL-1549 stage 1 with the implementation they exercised. |
| TC-016 | Compose complete scenario evidence without storing it | Property | P0 | FR-017, NFR-005 | 🚧 Planned — final outcome/resource types, deterministic partial resolution, per-axis distinctness and transient-value containment |
| TC-017 | Validate the consumer-owned delivery index | Property | P0 | FR-018, NFR-006 | ✅ Implemented — five executable index tests cover exact identity/state, bytes, typed absence, temporal/input/mutation/limit controls, bounded retention and version-lock non-authority. |
| TC-149 | Admit the checked-package handoff | Property | P0 | FR-043 | ✅ Implemented — eight tests admit recipe R's three packages, replay the standard mutation corpus, and check the retired-format, numeric, limit, identity-file and equality-file refusals. |
| TC-150 | Link from a typed read view of the checked package | Property | P0 | FR-044 | ✅ Implemented — the tests cover AC-1 to AC-7; windows are evaluation inputs and have no checked-package binding view. |
| TC-151 | Assess compensation through the checked-package evaluation | Property | P0 | FR-045 | 🚧 In progress — seven tests cover FR-045-AC-1, AC-3, AC-4, AC-6 and AC-8 (the Boolean, non-Boolean, undefined, refused, incomplete and family mappings are called directly, since no admitted input of R reaches the last four through a slot); AC-2, AC-5 and AC-7 are owed (see FR-045). |
| TC-152 | Consume temporal evidence through the in-memory evidence types | Property | P0 | FR-046 | 🚧 In progress — six tests cover FR-046-AC-1, AC-2, AC-3, AC-4 and AC-6 (five traced to those criteria, one tagged TC-152 only); AC-5 and AC-7 are owed (see FR-046). |
| TC-153 | End the retired compiler surfaces and the integrity lanes | Static | P0 | FR-047 | 🚧 In progress — three tests cover FR-047-AC-1, AC-2 and AC-4; AC-3 is owed (see FR-047). |
| TC-050 | Link role, queue, join and loop declarations | Property | P0 | FR-050, FR-051, FR-052 | 🚧 Planned — spec only (ADR-010); awaits the implementation. |
| TC-051 | Assess role lifetime and population bound | Property | P0 | FR-053 | 🚧 Planned — spec only (ADR-010); awaits the implementation. |
| TC-052 | Assess queue capacity, overflow outcome and order | Property | P0 | FR-054 | 🚧 Planned — spec only (ADR-010); awaits the implementation. |
| TC-053 | Assess join settlement per policy | Property | P0 | FR-055 | 🚧 Planned — spec only (ADR-010); awaits the implementation. |
| TC-054 | Assess loop bound and report proof as undischarged | Property | P0 | FR-056, FR-059 | 🚧 Planned — spec only (ADR-010); awaits the implementation. |
| TC-055 | Map QSL Unsupported and requires-bound outcomes to dispositions | Property | P0 | FR-057, FR-058 | 🚧 Planned — spec only (ADR-010); awaits the implementation. |
| TC-154 | Settle deadline branches and bind captures | Property | P0 | FR-100, FR-101 | 🚧 Planned — spec only (ADR-010); awaits the implementation. |
| TC-147 | Admit protocol refinement dispatch | Property | P0 | FR-301 | 🚧 Planned — binds all 6 FR-301 criteria; awaits the dispatch implementation; QSL implements FR-176/FR-177 (ADR-010). |
| TC-148 | Carry a trace-position witness on the protocol result wire | Property | P0 | FR-302 | 🚧 Planned — binds all 5 FR-302 criteria; awaits `agent-ix/quire-specification#114`'s `native-run-result/2`. |
| TC-160 | Dispatch local-projection and realizability claims to QSL's backend | Property | P0 | FR-150 | 🚧 Planned — binds all 9 FR-150 criteria; awaits the implementation (ADR-010). |
| TC-161 | Bind communication and visibility premises | Property | P0 | FR-151 | 🚧 Planned — binds all 6 FR-151 criteria; awaits the implementation (ADR-010). |
| TC-162 | Carry a projection or realizability witness joined to the linked graph | Property | P0 | FR-152 | 🚧 Planned — binds all 5 FR-152 criteria; awaits the implementation (ADR-010). |
| TC-163 | Settle projection and realizability claims unsupported while no backend serves the kind | Property | P0 | FR-153 | 🚧 Planned — binds all 7 FR-153 criteria; awaits the implementation (ADR-010). |

## Test Execution Summary

Row counts, recomputed from the tree. Every matrix row above is counted exactly
once, and `Blocked` means unexecutable — neither passing nor failing.

The table below counts matrix rows by the status their Status cell reads; it
does not measure binding.

| Category | Total | Passed | Failed | Blocked | Planned | Retired |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Functional requirement rows | 56 | 9 | 0 | 0 | 28 | 19 |
| Domain object requirement rows | 9 | 3 | 0 | 0 | 3 | 3 |
| Non-functional requirement rows | 8 | 3 | 0 | 0 | 4 | 1 |
| Integration rows | 4 | 0 | 0 | 1 | 0 | 3 |
| Test case rows | 35 | 9 | 0 | 0 | 18 | 8 |
| Overall | 112 | 24 | 0 | 1 | 53 | 34 |

The Integration count is four because IT-001 carries a separate version 1 and version 2
row; the two contracts may not substitute for each other, so one row cannot
report both. Both IT-001 rows and IT-002 are retired with their procedures (FR-047-AC-3 owns the replacement); IT-003 remains
unexecutable.

### Owed acceptance criteria

`NFR-005` denominates on what each lane **owes**, not on what it executed, so a
fully blocked integration cannot score 100%. These totals follow that rule: the
denominator is every authored acceptance criterion, and an unexecutable
criterion is a zero inside it rather than a row removed from it.

| Category | Total | Passed | Failed | Blocked | Planned | Retired |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Overall owed criteria | 430 | 134 | 0 | 0 | 163 | 133 |

Passed means the criterion carries a living traced test in the current tree
(134 of 430). Retired means the criterion's row lost every test in QSL-1549
stage 1 (133). Blocked is none: QSL delivered the evaluation entries that FR-045 and FR-046 wait on. Planned
(163) includes the criteria that verify by Inspection or Analysis only and
FR-014-AC-6, which waits on IT-003's Quoin scenario-record handoff.
