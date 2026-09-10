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
  - { target: ix://agent-ix/quire-protocol/NFR-001, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-002, type: references }
  - { target: ix://agent-ix/quire-protocol/NFR-003, type: references }
---
# Implementation Plan: Finite Native Quire Protocol Rust Slice

## Requirements Summary

### Stakeholder Requirement

- [ ] **StR-001**: Evaluate selected finite native Quire protocols without semantic loss.

### Functional Requirements

- [ ] **FR-001**: Admit one exact promoted protocol profile.
- [ ] **FR-002**: Link a finite typed protocol representation.
- [ ] **FR-003**: Assess compensation, retry, commit, and recovery state.
- [ ] **FR-004**: Assess bounded finite global conformance.
- [ ] **FR-005**: Dispatch protocol claims independently.
- [ ] **FR-006**: Produce and strictly read canonical protocol results.
- [ ] **FR-007**: Project canonical results to the selected E02 contract.
- [ ] **FR-008**: Return typed refusals through every public failure path.

### Non-Functional Requirements

- [ ] **NFR-001**: Reproduce semantic results and canonical bytes.
- [ ] **NFR-002**: Enforce every caller-supplied resource bound.
- [ ] **NFR-003**: Preserve the Rust and shared-assurance boundary.

## Dependency Graph

- `accepted native Quire candidate 1-draft.4 + accepted compiled-artifact contract -> FR-001`
  Reason: admission must bind exact promoted semantics and a closed typed meaning
  for compiler-produced bytes, including a lossless versioned numeric wire
  profile; candidate prose cannot serve as that interface.
- `FR-001 -> FR-002`
  Reason: only an admitted profile can be linked into the local representation.
- `FR-002 -> FR-003, FR-004`
  Reason: compensation and conformance consume the same identity-preserving finite graph.
- `FR-003 -> FR-004`
  Reason: global conformance reports compensation and recovery obligations.
- `FR-004 -> FR-005`
  Reason: independent claim dispatch preserves the completed finite-conformance result.
- `FR-004 + FR-005 -> FR-006`
  Reason: the canonical result contains both protocol outcome and per-claim dispositions.
- `FR-006 -> FR-007`
  Reason: E02 is derived only from a complete strict canonical source result.
- `FR-001..FR-008 -> NFR-001, NFR-002`
  Reason: reproduction and resource guards cover every public operation and identity path.
- `FR-002 + accepted compiler contract -> IT-001`
  Reason: compiler integration can run only after admission and linking exist.
- `FR-003 + FR-004 + FR-006 + accepted observation/temporal contracts -> IT-002`
  Reason: producer facts must reach compensation, conformance, and result construction unchanged.
- `FR-006 + FR-007 + accepted Engineering Assurance/Quoin contracts -> IT-003`
  Reason: typed transport, E02 reading, and retained evidence are external owners' boundaries.
- `IT-001 + IT-002 + IT-003 + NFR-001 + NFR-002 -> NFR-003 gate`
  Reason: the final unchanged-head review must cover every local and cross-repository path.

The finite graph, stable identity types, refusal envelope, and canonical JCS layer
are shared dependencies. Each is authored once on the serial core path and reused
by downstream integrations; no adapter may introduce another semantic model.

## Test Plan

### Property and Model Tests

- [ ] **TC-001** (FR-001 AC-1..AC-4): accept only exact promoted profile bytes and reject every one-axis identity, state, feature, unsafe-number, non-canonical exact-number, field/signed-64 overflow, invalid-rational, and cross-domain digest mutation.
- [ ] **TC-002** (FR-002 AC-1..AC-4): link every FR-050..FR-054 typed identity, legal interleaving, join, bounded progress/termination, channel premise, and owned visible choice; reject each invalid source mutation and discriminate semantic from presentation changes.
- [ ] **TC-003** (FR-003 AC-1..AC-7): discriminate trigger/capture/anchor activation, effects, replay-idempotent registration, commits, retries, full/partial recovery relations, population authority, missing inputs, and business-I/O purity.
- [ ] **TC-004** (FR-004 AC-1..AC-4): compare batch and incremental finite conformance over legal permutations, independently varied decision-scope/surrounding/global-conformance closure, and every adverse execution state.
- [ ] **TC-005** (FR-005 AC-1..AC-4): keep all requested claim dispositions independent and refuse unsupported or failed backends without fallback.
- [ ] **TC-006** (FR-006 AC-1..AC-6): enumerate canonical-result rows, strict reads, support topology, independent decision-scope/surrounding-execution progress and closure, decisive settlement, and immutable supersession.
- [ ] **TC-007** (FR-007 AC-1..AC-5): exercise the total FR-062-to-E02 map, finding preservation, derived identities, adequacy separation, and refusals.
- [ ] **TC-008** (NFR-001 criteria): prove byte identity across equivalent presentations and discrimination across semantic mutations.
- [ ] **TC-009** (NFR-002 criteria): probe below, at, and above every owner limit and kill each isolated guard-removal mutant.
- [ ] **TC-011** (FR-008 AC-1..AC-4): enumerate typed refusal fields, source codes, unknown versions, rendering independence, and partial-output rules.

### Static and Assurance Test

- [ ] **TC-010** (NFR-003 AC-1..AC-2 and measurements): enforce Rust-only executable paths, manual workflows, forbidden unsafe code, compiler-checked trace bindings, and shared assurance ownership.

### Integration Tests

- [ ] **IT-001 SC-01..SC-05** (FR-001, FR-002): consume the accepted native compiler artifact without a local parser, numeric loss, digest-domain substitution, or identity reinterpretation.
- [ ] **IT-002 SC-01..SC-04** (FR-003, FR-004, FR-006): consume accepted observation and temporal facts without duplicating their producer semantics.
- [ ] **IT-003 SC-01..SC-05** (FR-006, FR-007, NFR-003): publish typed results through Engineering Assurance, E02, and Quoin without a local runner, transcript parser, or evidence store.

## Remaining Work

### Track A: Critical Path (serial)

- **A1 = Task-001** Admit the selected profile and establish the crate — Hard; exit: exact promoted bytes and lossless numeric values admit through a Rust API while every identity or representation near-match refuses before evaluation.
- **A2 = Task-002** Link the finite representation — Hard; exit: all legal finite graphs link with exact identities and every construction-fatal defect returns no graph.
- **A3 = Task-003** Assess compensation state — Hard; exit: registration, retries, commits, and recovery follow only authored facts with no business execution.
- **A4 = Task-004** Assess finite conformance — Hard; exit: batch and settled incremental evaluation agree across every legal interleaving.
- **A5 = Task-005** Dispatch independent claims — Medium; exit: unsupported or failed claims cannot change any independently established result.
- **A6 = Task-006** Produce canonical results and refusals — Hard; exit: strict readers round-trip every valid row and reject every forbidden state or lineage.
- **A7 = Task-007** Project results to E02 — Hard; exit: the selected total map preserves all findings and admits no adequacy or sufficiency promotion.
- **A8 = Task-008** Enforce reproduction and resource bounds — Hard; exit: semantic permutations reproduce byte-for-byte and every resource guard discriminates.

### Track B: Accepted Producer Integrations

- **B1 = Task-009** Integrate the native compiler artifact — Medium; exit: the accepted producer artifact enters the public API with exact source identities and no alternate frontend.
- **B2 = Task-010** Integrate observation and temporal facts — Hard; exit: accepted facts reach compensation, conformance, and result fields unchanged.

### Track Gate: Shared Assurance and Qualification

- **Gate = Task-011** Integrate Engineering Assurance/E02/Quoin and run the unchanged-head gate — Hard; measures complete typed transport, traceability, language containment, Rust review, and semantic coverage; pass: every planned row is backed, all local gates pass, and every in-scope finding is closed.

## Parallel Execution Summary

```text
A1 -> A2 -> A3 -> A4 -> A5 -> A6 -> A7 -> A8
             \-> B1 -------------------------\
                              \-> B2 ----------+-> Gate
```

Task-009 may start after Task-002 and the compiler contract are accepted.
Task-010 may start after Task-006 and its external producer contracts are accepted.
All Cargo work remains serialized on the shared workstation even when source ownership permits parallel progress.

## Task File Mapping

| Task | Track | Owns (references) | Verified by (verifies) | Status |
| --- | --- | --- | --- | --- |
| Task-001 | A | FR-001 | TC-001 | blocked |
| Task-002 | A | FR-002 | TC-002 | blocked |
| Task-003 | A | FR-003 | TC-003 | blocked |
| Task-004 | A | FR-004 | TC-004 | blocked |
| Task-005 | A | FR-005 | TC-005 | blocked |
| Task-006 | A | FR-006, FR-008 | TC-006, TC-011 | blocked |
| Task-007 | A | FR-007 | TC-007 | blocked |
| Task-008 | A | NFR-001, NFR-002 | TC-008, TC-009 | blocked |
| Task-009 | B | FR-001, FR-002 | IT-001 | blocked |
| Task-010 | B | FR-003, FR-004, FR-006 | IT-002 | blocked |
| Task-011 | Gate | StR-001, FR-006, FR-007, NFR-003 | IT-003, TC-010 | blocked |

## Coordination Rules

- Accept A's proposed immutable candidate `1-draft.4` through the composed review and accept the native compiler's versioned compiled-protocol artifact contract, including its lossless numeric wire profile, tracked by [quire-spec-language#40](https://github.com/agent-ix/quire-spec-language/issues/40) before Task-001 writes source or dependency manifests; apply the selected `AGPL-3.0-or-later` policy to all authored implementation and dependencies.
- Keep native Quire grammar, temporal semantics, observation production, producer execution, evidence retention, and qualification disposition in their owning repositories.
- Freeze each accepted interface revision before its integration task and refuse unknown versions rather than adding compatibility fallbacks.
- One writer owns the shared representation, refusal, and canonical-result types; consumers merge after those contracts.
- Run `CARGO_BUILD_JOBS=1` under `/tmp/quire-heavy-check.lock`; do not trigger hosted CI, and keep every workflow manual-dispatch only.
- Finish on one unchanged head with the exact Rust review skill and semantic QUOIN `/gap-analysis`; remediation updates affected specs without restarting the full spec cycle unless the design changes materially.
