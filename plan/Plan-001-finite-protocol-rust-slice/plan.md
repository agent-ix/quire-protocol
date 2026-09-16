---
id: Plan-001
title: "Finite native Quire protocol Rust slice"
type: Plan
status: active
relationships:
  - { target: ix://agent-ix/quire-protocol/StR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-003, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-004, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-005, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-006, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-007, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-008, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-009, type: references }
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
# Implementation Plan: Finite Native Quire Protocol Rust Slice

## Requirements Summary

### Stakeholder Requirement

- [ ] **StR-001**: Evaluate selected finite native Quire protocols without semantic loss.

### Functional Requirements

- [x] **FR-001**: Strict `/1` and `/2` admission, mutual refusal and the `/1`
  two-revision compatibility regression are implemented.
- [ ] **FR-002**: Complete the exact version-2 binding and linked graph.
- [ ] **FR-003**: Connect the implemented compensation kernel to the linked graph.
- [ ] **FR-004**: Connect one batch/incremental conformance kernel to graph and compensation outputs.
- [ ] **FR-005**: Consume conformance through closed independent claim dispatch.
- [ ] **FR-006**: Assemble the implemented canonical result from typed subsystem outputs.
- [x] **FR-007**: Project validated results through accepted E02 draft-2.
- [ ] **FR-008**: Route every public failure through the shared exact-stage envelope.
- [ ] **FR-009**: Resolve every consumed external contract before its value is read.
- [x] **FR-042**: Strict canonical-result reader and complete borrowed Contract-IR map are implemented.

### Non-Functional Requirements

- [x] **NFR-001**: Complete whole-pipeline semantic and byte reproduction.
- [x] **NFR-002**: Complete one inventoried request accountant over every final path.
- [ ] **NFR-003**: Complete Rust/shared-assurance qualification without local infrastructure.
- [x] **NFR-004**: The exact single-lane local gate is executable.
- [ ] **NFR-005**: Compose and hand off every owed scenario record.
- [ ] **NFR-006**: Keep manifest, lockfile and rights record exact after every pin.
- [x] **NFR-007**: Keep `/1`, `/2` and external contract paths version-explicit.
- [ ] **NFR-008**: Carry every owed refusal or unexecutable axis in its returned value.

## Dependency Graph

- `Task-016 architecture gate -> every remaining implementation task`
  Reason: AD-001 fixes the complete type flow, owner direction, public surfaces,
  resource/refusal rules and external adapters once for the campaign.
- `Task-016 -> Task-017 -> Task-013 -> Task-002`
  Reason: the shared outcome/refusal foundation must exist before resolver
  failures; external values then enter version-explicit adapters only after the
  resolver mints request-bound constructor-private capabilities.
- `Task-013 -> Task-019 -> Task-009, Task-008`
  Reason: the consumer delivery index must prove that it cannot influence the
  resolver, then exist before complete delivery metadata is consumed and before
  the final source/reproduction sweep. Task-019 may run in parallel with the
  finite-linker serial core after Task-013.
- `Task-002 -> Task-003 -> Task-004 -> Task-005 -> Task-006`
  Reason: linked graph, compensation snapshot, conformance report, claim
  dispositions and canonical result assembly form the serial semantic core.
- `Task-002 -> Task-009`
  Reason: the `/1` integration must execute the real local linker, not admission alone.
- `Task-006 -> Task-007, Task-010`
  Reason: both E02 projection and observation/native-temporal integration require
  validated canonical result assembly; Task-010 consumes QSL/QObs, not QCI.
- `Task-010 -> Task-020`
  Reason: the completed observation/native-temporal assessment precedes its
  exact C00 contract/schema compatibility successor.
- `Task-007 + Task-009 + Task-010 + Task-013 + Task-019 -> Task-008`
  Reason: the final resource/reproduction sweep can close only after every public
  path and external-contract site exists.
- `Task-006 + Task-008 + Task-013 -> Task-018`
  Reason: scenario evidence is composed only from the final resolution,
  operation/claim outcome and resource types, never from an early duplicate.
- `Task-007 + Task-008 + Task-009 + Task-014 + Task-018 + Task-020 -> Task-011`
  Reason: unchanged-head qualification executes all local and external paths with
  resolved contracts and complete scenario evidence.
- `Task-015 -> QCI FR-025` is delivered; QCI #71 / PR #78 (`69ec82b`) is also
  delivered. QCI builds predicate valuations from Protocol leaf-result views,
  then joins independent QSL-native and TL formula results. No reverse Protocol
  dependency exists.

The shared dependencies are the constructor-private stage capabilities,
componentwise identity domain, typed refusal envelope, request accountant and
shared result-domain values. Each has one Protocol owner and no adapter-local copy.

## Test Plan

### Property and model tests

- [x] **TC-001**: exact versioned admission and all one-axis admission refusals.
- [x] **TC-002**: exact graph/binding identity, exhaustive typed content, FIFO,
  relationship replay/ambiguity, causal interleavings and fatal refusals.
- [x] **TC-003**: graph-derived compensation, activation, commit, retry, recovery,
  replay, transactionality and zero business calls.
- [x] **TC-004**: graph-derived batch/incremental agreement across legal semantic
  permutations, progress/closure and contradiction states.
- [x] **TC-005**: conformance-derived ordered claim dispositions, exact retained
  premises and no callback fallback.
- [x] **TC-006**: subsystem-derived canonical rows, strict reads, exact settlement,
  support topology and immutable corrections.
- [x] **TC-007**: total FR-062 E02 mapping, exact derived identity, strict target
  reading, finding preservation and no sufficiency promotion.
- [x] **TC-008**: whole-path byte/semantic reproduction and one-axis mutation discrimination.
- [x] **TC-009**: bidirectional final source inventory, below/at/over bounds,
  deadline checks, guard mutants and zero action calls.
- [ ] **TC-010**: shared Rust assurance checker, trace binding, EA execution and
  Quoin containment.
- [x] **TC-011**: exact-stage refusal envelopes, local/source causes, unsupported
  catalogs, partial-output rules and simultaneous causes.
- [x] **TC-012**: exact local gate policy and every independent gate defect.
- [x] **TC-013**: the resolver slice executes bounded two-state manifest
  resolution, private capability construction, interface authenticity and
  deterministic first-cause reporting; Task-019 owns the remaining
  delivery-index controls.
- [x] **TC-014**: consumer identity discipline.
- [x] **TC-015**: canonical reader and complete structural Contract-IR result view.
- [ ] **TC-016**: final scenario-evidence composition and cross-record refusal-axis distinctness.
- [x] **TC-017**: exact consumer delivery-index identity/state, typed absence,
  byte retention, binding/limit rules and version-lock non-authority.

### Integration tests

- [x] **IT-001 `/1` SC-01..SC-05 and SC-06a**: default-running admission plus
  local-link regression and before/after compatibility comparison. The `/2`
  SC-06..SC-09 handoff is already complete under Task-012.
- [ ] **IT-002**: Task-010's SC-01 through SC-04 traverse exact QObs/QSL owner
  values through activation, conformance, dispatch and canonical results.
  Task-020 executes SC-08's exact C00 contract/schema inventory and negative
  resolution controls. SC-05 through SC-07 remain planned.
- [ ] **IT-003**: validated Protocol results project to E02, execute through the
  Engineering Assurance boundary and hand a complete FR-017 value to Quoin.

## Remaining Work

### Architecture Gate

- **Architecture = Task-016** Freeze the complete implementation architecture —
  exit: AD-001, affected requirements, matrix, tasks and live tickets agree with
  current code and every selected review has no unresolved finding.

### Core Pipeline

- **Foundation = Task-017** Shared outcomes/refusals — exit: the one composed
  operation/claim outcome algebra, resource-incomplete value, exact stage/domain
  map and constructor-private refusal/source-cause foundation exist; Tasks
  002–006 consume them while replacing their standalone kernel transports.
- **Core-1 = Task-013** Version-lock enablement — exit: exact manifest
  resolution mints only bounded request-bound private capabilities.
- **Boundary index = Task-019** Consumer delivery record — exit: the exact
  FR-018 index executes while remaining unable to supply, repair or widen a
  version-lock entry. It may run in parallel with Core-2 after Task-013.
- **Core-2 = Task-002** Exact finite linker — exit: `/2` produces one complete
  typed graph or an exact fatal refusal with no approximate identity/content.
- **Core-3 = Task-003** Compensation bridge — exit: compensation consumes only
  linked/runtime values and commits replay-safe state transactionally.
- **Core-4 = Task-004** Conformance bridge — exit: batch and incremental frontends
  agree through one kernel derived from graph and compensation output.
- **Core-5 = Task-005** Claim bridge — exit: ordered independent dispositions
  consume conformance and unsupported claims invoke no fallback.
- **Core-6 = Task-006** Result/refusal composition — exit: canonical results and
  exact-stage refusals are constructible only from typed upstream outputs.

### Producer Integration

- **Native = Task-009 (done)** `/1` compatibility lane — the complete owner
  corpus runs by default through admission and linking, and SC-06a passes at
  both historical revisions around the `/2` addition.
- **Facts = Task-010 (done)** QObs/QSL native-temporal lane — SC-01 through
  SC-04 exact owner facts and adverse substitutions reach conformance, dispatch
  and canonical results unchanged.
- **C00 compatibility = Task-020 (blocked)** QObs candidate-interface successor —
  SC-08 resolves partial, activation, authority-bundle v1/v2 and lineage
  contract/schema selections locally at `581d98f`; draft owner PR #27 blocks
  promotion, and repair/query remain typed unsupported.
- **E02 = Task-007 (done)** Portable projection — every FR-062 row passes the
  exact accepted qverify strict reader.

### Qualification and Cross-cutting Closure

- **Bounds = Task-008 (done)** Final resource/reproduction sweep — every final
  public site has exactly one source-bound inventory/accountant/test path and
  equivalent result, producer-integration and E02 presentations reproduce.
- **Evidence = Task-018** Final scenario composition — exit: complete/partial
  FR-017 values and integration-level outcome distinctness use the final shared
  types and no local store.
- **Qualification = Task-011** EA/E02/Quoin unchanged-head gate — exit: every owed
  scenario is executed or counted unexecuted with complete typed evidence.

## Parallel Execution Summary

```text
Task-016 -> Task-017 -> Task-013 -> Task-002 -> Task-003 -> Task-004 -> Task-005 -> Task-006
                            |       |                          |          |
                            |       `-> Task-009               |          +-> Task-007
                            `-> Task-019                       `-> Task-010
Task-010 -> Task-020 -> Task-011
Task-007 + Task-009 + Task-010 + Task-013 + Task-019 -> Task-008 -> Task-018 -> Task-011
Task-015 (done) -> QCI FR-025; QCI #71/PR #78 done (external)
```

Source ownership allows the integration lanes to overlap after their gates, but
all heavy Cargo work is serialized with one job and the workstation lock. Plan
track names are subsystems and are not agent assignments; Agent B remains on TL.

## Task File Mapping

| Task | Track | Owns | Verified by | Status |
| --- | --- | --- | --- | --- |
| Task-001 | Core | FR-001 | TC-001 | done |
| Task-002 | Core | FR-002 | TC-002 | done; PR #67 / `2a3ef19` |
| Task-003 | Core | FR-003 | TC-003 | done; graph-owned Engine bridge and exact QSL/QObs evidence |
| Task-004 | Core | FR-004 | TC-004 | done; strict FR-300 activation-derived frontends share the graph/compensation kernel |
| Task-005 | Core | FR-005 | TC-005 | done; activation-derived Engine dispatch and closed local conformance backend |
| Task-006 | Core | FR-006, FR-008 | TC-006, TC-011 | done; atomic Engine orchestration, private result assembly, shared accounting and SR-116/SR-117 pass |
| Task-007 | Producer Integration | FR-007 | TC-007 | done; exact accepted qverify/EA graph, total source-bound projection and SR-118/SR-119 pass |
| Task-008 | Cross-cutting Closure | NFR-001, NFR-002 | TC-008, TC-009 | done; final inventory, one accountant, reproduction matrix and SR-120/SR-121 pass |
| Task-009 | Producer Integration | FR-001, FR-002 | IT-001 | done; default `/1` admission/static link and SC-06a two-revision comparison pass |
| Task-010 | Producer Integration | FR-003, FR-004, FR-006, FR-009 | IT-002 | done; SC-01 through SC-04 pass through real QObs/QSL owner paths; SR-120/SR-121 pass |
| Task-011 | Qualification | StR-001, FR-006, FR-007, NFR-003 | IT-003, TC-010 | blocked on Quoin scenario intake and remaining predecessors |
| Task-012 | Producer Integration | FR-001, FR-002, NFR-002 | IT-001 | done |
| Task-013 | Core | FR-009, FR-012, FR-016, NFR-006 | TC-013 | done; PR #50, SR-103/SR-104 pass |
| Task-014 | Qualification | NFR-004 | TC-012 | done |
| Task-015 | Producer Integration | FR-042 | TC-015 | done |
| Task-016 | Architecture Gate | AD-001, FR-001..FR-009 | TC-002..TC-013 | done; SR-089..SR-100 pass under #61 |
| Task-017 | Cross-cutting Foundation | FR-008, FR-014, FR-016 | TC-011 | done; 30 focused tests and SR-101/SR-102 pass under #62 |
| Task-018 | Qualification | FR-017, NFR-005 | TC-016 | planned |
| Task-019 | Producer Integration | FR-018, NFR-006 | TC-017 | done; exact transactional bounded consumer index cannot affect version-lock resolution; SR-108/SR-109 pass |
| Task-020 | Producer Integration | FR-009, NFR-006 | IT-002 | blocked; SC-08 and the enforced pre-assessment resolver pass locally at exact candidate `581d98f`, but QObs PR #27 remains draft and blocks Protocol promotion |

## Coordination Rules

- AD-001 and its whole-scope review are the architecture authority for all
  remaining tasks. No later ticket-scoped `/specify` or `/spec-review` is owed
  for this unchanged scope.
- Consume only accepted owner types and merged revisions. Missing or incompatible
  contracts yield an exact blocker/refusal, never a local mirror.
- QObs facts flow into Protocol; Protocol leaf-result views flow into QCI's
  accepted FR-025 predicate bridge, which supplies valuations to QCI #71's
  native/TL join. Protocol does not import that downstream join.
- Engineering Assurance executes, Protocol composes and passes scenario values,
  and Quoin retains. Protocol adds no runner or evidence store.
- Each owning task adds its resource sites; Task-008 performs the final global
  reconciliation after all public paths land.
- Use focused tests during red/green work. Run the complete local gate once the
  ticket candidate is ready, with `CARGO_BUILD_JOBS=1`, the workstation lock and
  `--target-dir target-codex-backends`; never dispatch hosted CI.
- At PR readiness run self `/rust-review` and `/gap-analysis`, fix every finding,
  admin merge, then immediately update the task, ticket, epic, matrix, plan,
  dependency ledger and exact evidence revision.
