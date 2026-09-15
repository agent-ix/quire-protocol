---
id: SR-106
title: "Rust review of the Task-003 graph-owned compensation engine"
type: SpecReview
analysis: code-review
scope: "Task-003 / quire-protocol#5 exact-evidence completion over d768a4d"
review_set: subset
relationships:
  - { target: ix://agent-ix/quire-protocol/Task-003, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-003, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-019, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-020, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-021, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-022, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-023, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-024, type: references }
  - { target: ix://agent-ix/quire-protocol/FR-025, type: references }
  - { target: ix://agent-ix/quire-protocol/TC-003, type: reviews }
---
# SR-106: Rust Review of the Task-003 Graph-Owned Compensation Engine

## Summary

Reviewed the complete Task-003 Rust, tests and tracking against issue #5 and
FR-003/FR-019..FR-025 after PR #72 advanced `origin/main` to `d768a4d`. The
review retained PR #72's stage-specific assessment refusals and private result
composition boundary while closing the exact-evidence gaps in its compensation
snapshot. Every in-scope finding was fixed before this artifact was finalized.

## Verdict

**PASS for Task-003 / quire-protocol#5.** No unresolved Rust, owner-value,
replay, identity, panic, unsafe, test-quality, trace or local-gate finding
remains in this subset. Task-008 still owns the plan-wide resource-accounting
inventory and final one-short qualification; this review does not pre-credit it.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Resolved: public `registration::Snapshot::from_inputs` let consumers mint registrations before the admitted guard completed true. The bypass constructor was removed; only the Engine's transactional activation path can retain a registration. | FR-003-AC-2/7; FR-020-AC-1/3 | implementation-bug-despite-evidence |
| FND-002 | high | Resolved: a true guard discarded its QSL report, retained no evaluated state view, and collapsed a missing forward effect to a report-free marker. Activation now retains the exact view/report, changed replay conflicts, and post-evaluation unavailability returns the complete report without mutation. | FR-003-AC-1/6/7; FR-019; FR-020 | implementation-bug-despite-evidence |
| FND-003 | high | Resolved: graph attempts omitted FR-235's independently established related effect and accepted retry replay after comparing only registration/outcome. Attempts now retain the optional established effect plus exact retry view/report; an unretained effect is unavailable and changed retry evidence conflicts transactionally. | FR-003-AC-3/4/7; FR-022; FR-023 | implementation-bug-despite-evidence |
| FND-004 | high | Resolved: a completed-false retry was reduced to `RetryNotAuthorized` and lost QSL limits, usage and Boolean verdict. The boxed complete report now travels in the typed refusal and a killing assertion inspects its false value. | FR-003-AC-6/7; FR-022-AC-3/4; FR-024-AC-6 | implementation-bug-despite-evidence |
| FND-005 | high | Resolved: recovery and ordering retained derived outputs without the exact QSL operand view or QObs observation/position views, so changed owner evidence could compare as replay. The graph now retains and compares every owner view with the report/violations before declaring replay. | FR-003-AC-3/5/7; FR-021; FR-024 | implementation-bug-despite-evidence |
| FND-006 | medium | Resolved: changed recovery or ordering evidence could overwrite/append after a prior accepted result rather than refuse a conflicting replay. Candidate snapshots now compare first and commit only new or exact-replay content; tests assert prior-state equality on every conflict. | FR-003-AC-7; FR-021-AC-4; FR-024-AC-2 | implementation-bug-despite-evidence |
| FND-007 | medium | Resolved: duplicate public `Engine`/`AssessmentBundle` attempt/effect vectors stayed empty after graph admission and could report a false state alongside the authoritative graph snapshot. The stale vectors/accessors were removed; graph state is the sole public result. | FR-003-AC-4; FR-023 | implementation-bug-despite-evidence |

## Rust Review

| Check | Result |
| --- | --- |
| Ownership and public surface | `Engine` is the only mutation owner. Graph records expose read-only accessors; `Snapshot::default` can create only an empty value and every mutator/registration constructor is crate-private. No callback, parser, transport or action capability was added. |
| Identity and equality | Registrations retain the complete FR-288 definition identity, declaration-local compensation handle and QObs FR-287 forward-effect subject. Attempt/effect keys remain distinct authority-qualified subjects. Commit, recovery and ordering remain bound to the exact registration. |
| Exact source outcomes | Guard, retry and recovery call QSL's pinned `/2` evaluator directly. Completed false, incomplete, refused, exhausted and non-Boolean outcomes stay distinct; no branch coerces one to success. QSL views/reports and QObs ordering views survive bundle assembly. |
| Replay and transactionality | Every mutating operation builds a candidate snapshot. Exact owner evidence returns `Replay`; a changed view, report, relationship, disposition, commit subject, recovery result or ordering result returns a typed non-success and preserves the previous snapshot. |
| Bounds and conversions | The initial attempt counts, checked addition prevents host overflow, signed-64 comparison widens to `i128`, and the authored positive finite maximum is enforced before retention. QSL and QObs inputs arrive through their owner-bounded types. Task-008 owns the independent aggregate accountant and is not claimed here. |
| Panic, unsafe and concurrency | Changed production code adds no `panic!`, `unwrap`, unchecked indexing, `unsafe`, recursion, async/blocking bridge, process, thread or lock. Test panics are fixture/assertion failures only. The large retry report is boxed, satisfying Clippy's enum/error-size checks without discarding content. |
| Test quality | Eight retained pure-kernel tests and two graph-Engine tests exercise activation, registration, all three ordering violations, exact/one-over retry, false/source outcomes, related-effect retention, full/partial recovery, exact and conflicting replay, missing evidence, commit-never, bundle retention and zero action calls. The tests inspect values and prior snapshots rather than only variant spelling. |
| Repository constraints | Rust only, `publish = false`, worktree-local `target-codex-backends`, no hosted CI, and no change under `resources/native-v1/`. The exact QSL pin remains `c0deb7b3ab66d6e8cb64a0fd7769bd0af82b61b7`. |

## Verification

The final committed candidate passes the repository-defined local gate in the
required order with Rust/Cargo 1.98.1, `CARGO_BUILD_JOBS=1`, the shared heavy
lock, `--locked`, and `--target-dir target-codex-backends`: format check,
all-target Clippy with warnings denied, then the complete test suite. Focused
assessment, TC-003 and closed-dispatch tests also pass after reconciling PR #72.
Quire 0.32.0 / engine `a874fb64` validates the artifacts and reports no
Task-003/TC-003 relevant unmatched tag or unbacked row. Exact head and command
output are retained in the pull-request evidence.

## Boundary

This PASS covers Task-003's graph/runtime compensation bridge only. It does not
claim Task-004 conformance integration, Task-008's final resource inventory,
the remaining Plan-001 tasks, or the whole epic complete.
