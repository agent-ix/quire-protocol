---
id: SR-107
title: "Gap analysis of the Task-003 graph-owned compensation engine"
type: SpecReview
analysis: gap-analysis
scope: "Task-003 / quire-protocol#5, FR-003/FR-019..FR-025, TC-003, Plan-001 and TM-001"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-003, type: reviews }
  - { target: ix://agent-ix/quire-protocol/Plan-001, type: reviews }
  - { target: ix://agent-ix/quire-protocol/TM-001, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-003, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-003, type: reviews }
---
# SR-107: Gap Analysis of the Task-003 Graph-Owned Compensation Engine

## Summary

Task-003's allocation is complete: the production Engine consumes the linked
QSL `/2` graph, exact authority-qualified QObs facts and QSL state views; owns
all compensation transitions; preserves every successful input/output needed
for later inspection; and refuses or returns all non-successes without partial
mutation. The matrix, task, plan, chunk register, log and issue #5 now describe
the implemented surface rather than the pre-review approximation.

## Verdict

**PASS for Task-003 / quire-protocol#5.** No in-scope task, matrix, trace,
semantic, reverse-code, stub or validation gap remains. This is not a PASS for
Task-004, Task-008, Plan-001 or epic #14 as a whole.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: the implemented graph result did not retain successful guard inputs/reports, attempt-to-effect relationships, retry inputs, recovery operands or ordering authority views even though TM-001 claimed exact retention. The graph and assertions now carry every value. | FR-003; FR-019..FR-024; TC-003; TM-001 | correct-requirement-no-evidence |
| FND-002 | high | Resolved: replay assertions killed changed attempt outcome and effect disposition but not changed guard limits, retry views/reports, commit subject, recovery view/report or ordering evidence. Each independent replay axis now has a transactional killing assertion. | FR-003-AC-7; FR-020-AC-3; FR-021-AC-4; FR-022-AC-3; FR-024-AC-2 | correct-requirement-no-evidence |
| FND-003 | medium | Resolved: a public registration constructor and two permanently empty compatibility vectors were production code without an owning requirement and contradicted the Engine-owned lifecycle. They were removed rather than assigned false trace credit. | `registration::Snapshot`; `Engine`; `AssessmentBundle` | implementation-bug-despite-evidence |
| FND-004 | medium | Resolved: TM-001's live coverage paragraph still reported the older Task-013 checkpoint after later mainline work. It now records the measured Task-003 checkpoint: 149/342 rows backed and 265/271 Rust candidates tagged/bound, while leaving the six wider untagged candidates visible. | TM-001; `quire coverage --scope . --json` | correct-requirement-no-evidence |

## Target Selection

- Target task: `plan/Plan-001-finite-protocol-rust-slice/tasks/Task-003-assess-compensation-state.md`.
- Ticket: quire-protocol#5, current body controlling; parent epic #14.
- Specification: FR-003 and its atomic parts FR-019 through FR-025.
- Evidence: TC-003, TM-001, the two graph-owned tests, the eight retained pure
  kernel tests, and the production modules they execute.
- Baseline: `origin/main` at `d768a4da1122ccdbbf5cc54e1870b4d411d4cc60`
  after PR #72; the candidate preserves its Task-006 orchestration changes.

## Plan and Matrix Reconciliation

| Task-003 obligation | Executed evidence | Result |
| --- | --- | --- |
| Graph/runtime-only construction | Engine derives the linked protocol, eligible effect pairs and definition identity; registration mutation is crate-private | Complete |
| Activation and registration | Exact QSL guard view/report, false/source/unsupported/missing paths, one registration per complete key and changed-replay refusal | Complete |
| Attempts and finite retry | Distinct QObs attempt keys, optional established-effect relationship, exact FR-236 outcome/absence, initial-count rule, true/false/source retry and exact/one-over bound | Complete |
| Commit and causal ordering | Exact selected commit/commit-never handling, all three typed violations, QObs record/position authority, missing positions and replay conflict | Complete |
| Recovery | Exact QSL recovery handle, operand view/report retention, missing effect/commit, resource/source outcomes, full failure beside partial success and callback compile-fail | Complete |
| Transactionality and purity | Candidate snapshots for every mutation, prior-state assertions for non-successes, capability mutation controls and zero business/compensation calls | Complete |
| Tracking | Task-003, Plan-001, chunk register, log, TC-003/TM-001 and ticket #5 agree with the implemented surface | Complete |

Task completion is 1/1 for this allocation. Task-004 is the next serial core
consumer; Task-008 remains the cross-cutting final accountant/inventory owner.

## Coverage and Trace Matrix

- `quire coverage --scope . --json`: 149/342 declared rows backed.
- Rust census: 271 candidates, 265 tagged, 265 bound.
- Task-003/TC-003 relevant unmatched tags: 0.
- Task-003/TC-003 relevant unbacked rows: 0.
- FR-003-AC-1..AC-7 and every FR-019..FR-025 criterion are bound to a
  default-running TC-003 symbol.
- The six repository-wide untagged candidates and the reported TC-014 status
  lie are outside Task-003 and remain visible; neither is used to claim this
  subset complete.

The optional gap-analysis semantic mode was not invoked because it requires a
separate explicit opt-in. The mandatory gap pass still compared each tagged
assertion and production path to its owning criterion, and SR-106 records that
code-level semantic review.

## Reverse-Gap and Stub Audit

The reverse inventory covered registration key/snapshot, activation, attempt,
effect, commit, recovery, ordering, missing/refusal/transition/outcome enums,
Engine mutation methods and bundle accessors. Each maps to FR-003 or one of
FR-019..FR-025. The remaining algebra/recovery output vocabulary is retained
for the eight pure TC-003 kernels; the former recovery evaluator trait and
function are crate-private and cannot enter production.

Searches over the changed production and test surface find no `todo!`,
`unimplemented!`, `dbg!`, placeholder success, ignored Task-003 test or unsafe
block. Production has no newly added panic-capable input path. Narrow lint
expectations retained from PR #72 name their Task-006 purpose and are outside
the compensation graph API.

## Validation

The final committed candidate passes the exact local gate in order: format
check, locked all-target Clippy with warnings denied, then the complete locked
test suite, all with one Cargo job, the shared heavy lock and worktree-local
`target-codex-backends`. Quire validates specification, plan and review
artifacts. No hosted run occurred, `resources/native-v1/` is untouched, and the
QSL dependency remains exactly pinned to
`c0deb7b3ab66d6e8cb64a0fd7769bd0af82b61b7`. Exact head and command output are
retained in the pull-request evidence.
