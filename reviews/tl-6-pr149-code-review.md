---
id: SR-2127041
title: "TL-6 PR #149 code-review (rust-review lane)"
type: SpecReview
analysis: code-review
review_set: subset
scope: "agent-ix/quire-protocol; TL-6; PR #149 head ecdac92; src/deadline.rs, src/captures.rs, src/closure.rs, src/lib.rs, tests/tc_154_settle_deadline_captures.rs"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
  - { target: ix://agent-ix/quire-protocol/FR-101, type: reviews }
---
## Summary

Ticket: TL-6. Code review with the Rust lane (rust-review, rust-style loaded) folded in, scoped to `git diff origin/main...HEAD` of PR #149 at head ecdac92. Focused checks run on the head: `cargo clippy --locked --all-targets -- -D warnings` clean; `cargo test --locked --test tc_154_settle_deadline_captures` 12 passed, 0 failed. No full gate run.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-1 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-2 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-3 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-4 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-5 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-6 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-101 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-3 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-4 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)
- FR-101-AC-5 (spec/functional/FR-101-bind-obligation-captures-as-ordered-atomic-occurrences.md)

## Verdict

**FAIL — not mergeable.** The response/timeout kernel, the inclusive deadline, the `closed`+`complete` timeout rule, the incomplete-over-open precedence, per-instance keying and the workflow-identity refusal (non-empty UTF-8 <= 4096 bytes via QSL `WorkflowInstanceIdentity::new`) are correct, and batch `assess` agrees with incremental `Wait::admit` by construction (it sorts by `ObservationRef` and replays through the same `admit`). A late matching response observed at or before the deadline supersedes the timeout under FR-033 and leaves the earlier result unaltered; one observed after the deadline supersedes nothing. `captures.rs` binds QSL slot outcomes unchanged, evaluates nothing, and refuses the whole activation listing every failing capture. `TruthDisposition` mirrors the FR-242 labels exactly, following the FR-241 `AssessmentExecution` precedent (no Rust crate publishes either). No vendoring, shim, new pin or `/// Trace:` line. Two lineage defects block merge: a late non-matching observation after timeout silently replaces the timeout result, and a later progress/completeness record contradicting a settled timeout is refused instead of producing an FR-033 invalidating result.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | A false or valueless observation observed at or before the deadline and admitted after timeout was selected changes the current result from Timeout to no-selection/`unavailable` (judge's `blocked` path) and pushes a new result with no `Supersession`: `supersedes` only fires when the new result is a Response. That is a silent replacement of a settled result, which the FR-100 bullet ('When an observation observed at or before the deadline is admitted after timeout was selected ... SHALL emit a superseding or invalidating result under FR-033') and FR-033 forbid. Emit an invalidating Supersession whenever a Timeout result is replaced, and add a test. | src/deadline.rs:455-484 |
| FND-002 | high | `WaitRefusal::PremiseRegressed` refuses a later authority fact whose progress or completeness no longer supports a selected timeout, and `assess` then aborts the whole batch. FR-033 (whose Inputs name 'the later observation, progress or closure record that contradicts a retained premise') and QSpec FR-061 ('A contradicted retained premise is reported by supersession') require a typed invalidating result naming the earlier result, the contradicted premise and the corrected input. FR-100 being silent does not license a refusal: the owner rule applies. Replace the refusal with an invalidating Supersession. | src/deadline.rs:440-449 |
| FND-003 | medium | `WaitKey::new` validates only the workflow identity. The await control (`CheckedNodeId`) and the repeat-ordinal count are not bound against QSL's `ProtocolOccurrenceSchema`, although that schema carries `ControlOperator::Await` control entries and `dynamics::bind_occurrence` (src/dynamics.rs:92-121) already binds join/loop occurrences with `schema.bind(identity, control, &owner, repeat_ordinals)`. An unknown control, a non-await control or a wrong ordinal count keys a wait. Reuse `bind_occurrence` (it also removes the duplicated UTF-8 conversion). | src/deadline.rs:74-91 |
| FND-004 | medium | The deadline is a caller-supplied `TemporalBoundary` of which only the upper bound is used. `admits` takes a raw `i128` with no clock-family or lower-bound check, so a response observed before the window's lower bound selects the response branch, and `named_by` ignores lower/carrier/watermark (`..`) and compares the upper bound by `to_string()` text equality, so a progress view for another boundary with the same upper bound passes. The checked clause `await ... within [lo,hi]` has a QSL `CheckedDeadline {lower, upper}`; read the deadline from the checked package if QSL publishes it, else ticket that, and compare the whole boundary. | src/deadline.rs:112-180 |
| FND-005 | low | Supplies are paired with authored captures by position, so one missing supply in the middle reports every later capture as `OutOfOrder` plus a surplus `Unauthored`, misnaming the captures that failed. The refusal is still correct; match supplies to authored captures by `CheckedNodeId` so `Missing` names the one capture QSL omitted. | src/captures.rs:172-213 |
| FND-006 | low | `Settlement` (and `Supersession`, `ResultId`, `WaitAssessment`) expose every field as `pub`, so a caller can construct a settlement with both a selected branch and a truth, contrary to the documented invariant 'A selected branch carries ... no truth'. Keep fields private with accessors, as `SettledResult` already does. | src/deadline.rs:297-309 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-007 | low | `AwaitDeclaration.clock` is read from the checked node but never used. `Deadline::named_by` accepts a progress boundary of any family (EventPosition, FixedSample or TimestampedEvent) and compares only the two numbers, and `Deadline::of` adds the authored bounds to the anchor with no check that the boundary is in the await's clock. A progress view on another clock family with equal numbers passes. Check the boundary family against the await's clock, or ticket it if QObs does not expose the clock identity. | src/deadline.rs:139-157 |
| FND-008 | low | `Deadline::of` computes `anchor + i128::from(declaration.lower)` and `anchor + i128::from(declaration.upper)` on a caller-supplied `anchor`. An anchor near `i128::MAX` overflows: a panic in debug builds and a wrapped, wrong window in release. Use `checked_add` and refuse. | src/deadline.rs:124-130 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-009 | low | FR-100 and AC-13 refuse a window that does not fit 'the clock's integer range', but `Deadline::of` checks only `i128` overflow. Event-position and fixed-sample boundaries are `u64`, so a negative window or one above `u64::MAX` on those clocks is accepted. The test covers only `i128::MAX`. Either say `i128` in the spec, or check the range of the clock's boundary family. | src/deadline.rs:137-149 |

## Dispositions

Round 1, fix commit 183d4f1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 183d4f1 |
| FND-002 | fixed | 183d4f1 |
| FND-003 | fixed | 183d4f1 |
| FND-004 | deferred | Fixed except the anchor. 183d4f1 reads the window from the checked await (`Deadline::of(declaration, anchor)` from `AwaitDeclaration{lower, upper}`), `admits` checks both bounds (AC-10) and `named_by` compares both bounds (AC-11). The anchor instant stays caller-supplied; FR-100 now states it as an Input ('the anchor instant') and the Description says the window is counted from the awaited `after` event. Acceptable for this PR because a progress view through a different window is refused, but a response settled before any authority fact uses the caller's anchor unchecked. Needs a ticket: derive the anchor from the admitted `after` observation. No ticket exists yet. |
| FND-005 | fixed | 183d4f1 |
| FND-006 | fixed | 183d4f1 |

Round 2, fix commit fb68003.

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | deferred | TL-388 (FR-100: derive the await window anchor from the admitted after-event observation). FR-100 Inputs now cite it: 'The anchor instant is supplied by the caller; reading it from the admitted history is carried by TL-388.' |
| FND-007 | fixed | fb68003 |
| FND-008 | fixed | fb68003 |

Round 3, fix commit 89325f1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-009 | fixed | 89325f1 |
