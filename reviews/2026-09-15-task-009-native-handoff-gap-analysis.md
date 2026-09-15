---
id: SR-111
title: "Gap analysis of the Task-009 native compiler handoff"
type: SpecReview
analysis: gap-analysis
scope: "Task-009 / quire-protocol#11, FR-001, NFR-007, IT-001, Plan-001 and TM-001"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-009, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/NFR-007, type: reviews }
  - { target: ix://agent-ix/quire-protocol/IT-001, type: reviews }
---
# SR-111: Gap Analysis of the Task-009 Native Compiler Handoff

## Summary

Task-009's allocation is complete: the exact owner-published `/1` corpus runs
by default through explicit admission and static linking, `/1` and `/2` remain
mutually refusing, all inherited selection fields preserve refusal semantics,
and the actual pre/post-`/2` revision pair passes the same historical corpus.
Task, matrix, master status, dependency evidence and issue #11 now agree.

## Verdict

**PASS for Task-009 / quire-protocol#11.** No in-scope task, matrix, semantic,
reverse-code, stub, tracking or validation gap remains. This is explicitly not
a PASS for the unfinished Plan-001 or epic #14.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: TM-001 marked NFR-007-AC-7 complete from the historical `/1` rerun without directly comparing the earlier and later readers over every inherited selection field. The new paired test mutates all eight inherited `Expected` fields, requires both versions to refuse, requires exact typed-error equality, and keeps temporal axes separate. | NFR-007-AC-7; IT-001-SC-06a; `tests/it001_v2_handoff.rs` | correct-requirement-no-evidence |
| FND-002 | medium | Resolved: the mutual-refusal test's trace attribute was not bound by the installed scanner, leaving FR-001 at 4/5 despite real execution. The narrowed four-ID attribute binds FR-001-AC-5 and NFR-007-AC-3/AC-4; measured coverage is now 153/342 and FR-001 is 5/5. | FR-001-AC-5; NFR-007-AC-3/AC-4; TM-001 | correct-requirement-no-evidence |
| FND-003 | medium | Resolved: the master specification and README still named the old QSL/QObs pins, treated Task-009/Task-019 as outstanding and named TC-016 as the highest test. They now record QSL `6c8c401`, QObs `93bc565`, completed Task-009/Task-019, TC-017 and the current coverage census. | `spec/spec.md`; `README.md`; Task-009; TM-001 | correct-requirement-no-evidence |

## Target Selection

- Target allocation: `plan/Plan-001-finite-protocol-rust-slice/tasks/Task-009-integrate-native-compiler.md`.
- Ticket: quire-protocol#11, current body controlling; parent epic #14.
- Specification root and matrix: `spec/`, `spec/tests.md` (`TM-001`).
- Identity prefix: `ix://agent-ix/quire-protocol`.
- Requirement scope: FR-001, NFR-007 and IT-001 SC-01 through SC-06a.
- Implementation/evidence: `src/intake/`, `src/model/v1.rs`, `src/inventory/map.rs`, `tests/it001_*`, QSL's pinned handoffs and the recorded historical revision pair.

## Plan and Matrix Reconciliation

| Task-009 obligation | Evidence | Result |
| --- | --- | --- |
| Exact owner dependency | Cargo, lockfile, dependency inventory and documentation all name QSL `6c8c401`; the resolved graph contains one QSL package | Complete |
| Default `/1` admission | 24 tests load `PUBLISHED_V1_HANDOFF`, build B's independent expectation first, and call `intake::v1::admit` with no environment selector | Complete |
| Static link | The admitted owner package links declaration 6 to exactly two roles, one channel, 16 controls and 36 causal edges | Complete |
| Refusal and numeric boundaries | Artifact, producer, feature, binding, baseline, digest-domain, numeric spelling/kind/range and exact-rational controls refuse without repair | Complete |
| Cross-version isolation | Both ordered cross-version submissions refuse distinctly; all eight inherited `Expected` fields refuse identically through `/1` and `/2` while temporal axes remain separate | Complete |
| SC-06a revision comparison | The identical 23-case test and selection blobs pass at Protocol `0e5ec292` and `5ce9070e`; the root facade and QSL `/1` public refusal surface are unchanged | Complete |
| Tracking | Task-009, Plan-001, TM-001, master status, README, dependency evidence and ticket #11 describe the same delivered boundary | Complete |

Task completion is 1/1 for this allocation. Across the wider Plan-001 bundle,
11 of 19 tasks are complete; the remaining tasks stay in progress, planned or
blocked under their own tickets and prevent a whole-plan PASS.

## Coverage

- Reconciliation: `quire coverage` 0.32.0 / engine `0.46.0@a874fb64`.
- Rows backed by a tagged symbol: 153/342.
- Rust census: 272 candidates, 266 tagged, 266 bound.
- FR-001: 5/5 criteria engine-bound.
- NFR-007: current executable AC-3, AC-4 and AC-7 engine-bound; AC-5 is the
  required two-revision execution record, and AC-1/AC-2/AC-6/AC-8 use their
  declared inspection or analysis methods.
- In-scope unbacked matrix test-case rows: 0; in-scope status lies: 0.
- The installed model cannot mint bullet-form IT scenario IDs pending
  `quire-rs#244`; TM-001 therefore reports integration-only evidence separately
  and does not present SC-06a as a persistent single-run test.
- Repository-wide gaps such as TC-014's pre-existing status lie and unfinished
  later tasks remain visible in the report and are not used to qualify this
  subset.

## Semantic Review

The user opted into self-review, so the optional intent-to-test-to-code pass ran
over FR-001's five criteria, NFR-007's eight criteria and IT-001 SC-01 through
SC-06a. Admission reaches the real QSL readers, linking reaches the real
Protocol linker, every mutation has an observable refusal or exact value, and
the two-revision requirement uses two actual historical builds rather than a
fabricated current-run proxy. No criterion is satisfied by a test double,
display string, locally mirrored owner type, inferred version, approximation or
unchecked success.

## Reverse-Gap and Stub Audit

The reviewed behavior inventory covered owner handoff loading, independent
selection, explicit `/1` admission, static linking, cross-version refusal,
inherited-axis parity, positive dependency/source inventory and historical
compatibility evidence. Each behavior maps to FR-001, NFR-007 or IT-001. The
sole production-code edit changes an internal call from the root compatibility
facade to its exact `intake::v1` target and introduces no public behavior.

Searches over changed Rust find no `todo!`, `unimplemented!`, `dbg!`, ignored
test, placeholder return, unsafe block or production panic path. All 45 focused
integration/inventory tests execute real code and contain discriminating
assertions; untraced behaviors, source stubs and test stubs are all zero.

## Validation

Quire validation reports all 236 specification, plan and review documents
grammar-clean. The focused local Rust lanes pass 3 dependency-inventory, 24
version-1 and 18 version-2 tests with zero failure or ignore. The
repository-defined full unchanged-head gate follows the review commit and is
recorded in the pull request.
