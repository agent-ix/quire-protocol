---
id: SR-2127052
title: "TL-388 PR #150 gap-analysis"
type: SpecReview
analysis: gap-analysis
review_set: subset
scope: "agent-ix/quire-protocol; TL-388; PR #150 head 98e95d0; spec/functional/FR-100, spec/test-cases/TC-154, spec/tests.md, src/deadline.rs, src/declarations.rs, tests/tc_154_settle_deadline_captures.rs"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
---
## Summary

Ticket: TL-388. Planless gap analysis (Plan completion: not assessed) of FR-100 against the computed Test Matrix (`quire matrix --format tsv`, quire 0.38.3, on head 98e95d0), the tagged tests and the code. FR-100-AC-1..AC-18 are each `tagged` to one TC-154 test; the hand-kept "all 13 criteria" figure in spec/tests.md is removed and the row now names only the AC range and TC-154.

## Examined

- FR-100-AC-13 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-14 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-15 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-16 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-17 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-18 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)

## Verdict

**PASS WITH FINDINGS.** No stubs and no coverage inflation; every new test asserts concrete branches, truths and refusals and can fail. Two gaps: the fixed-sample anchor path has neither a criterion nor a test, and the library ignores the `after` target QSL already publishes on the checked `await`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | The Behavior bullet "take the anchor instant ... the sample index for a fixed sample" and AC-13's fixed-sample range have no test: the `FixedSample` arms of `AfterAnchor::of` and `AfterAnchor::is_of` are never reached (AC-13 now tests only event positions and timestamps; the old shared unsigned-range check no longer covers fixed samples). Reading `epoch_nanos` instead of `index`, or swapping the `FixedSample` and `EventPosition` arms of `is_of`, survives every test. Fix in this PR: one fixed-sample `after` fixture with a fixed-sample family, in the AC-14 and AC-13 tests. | src/deadline.rs:240-242, 265; tests/tc_154_settle_deadline_captures.rs:704-751 |
| FND-002 | medium | QSL publishes the `after` anchor of every checked `await` as `after = {kind, target}` (qsl-package protocol_view.rs `awaiting`, kinds `event` and `compensation`, emitted as an `AwaitAnchor` edge), but `AwaitDeclaration` reads only `within` and the clock. The library therefore cannot name which checked event, or which compensation's forward body, the caller must attribute the `after` observation to, and a `compensation`-kind anchor is not handled or mentioned. Reading the target and kind is in-repo work on an owner-published fact; attributing a QObs observation to that node and workflow instance is a separate owner gap. | src/declarations.rs:155-171, 552-570; src/deadline.rs:380-394 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | The AC-23 test asserts only that `after.port` is `Exit` and `after.node` is not the await. Dropping the `EdgeKind::AwaitAnchor` filter in `await_control` would pick the sequence predecessor's exit edge into the await's Enter, which also passes. Assert the node equals the event control the fixture's `after` names. The `compensation` kind (anchor at the forward body's Enter) has no test. | tests/tc_154_settle_deadline_captures.rs:996-1003; src/declarations.rs:572-578 |

## New findings (disposition pass 2)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | In the event fixture `Armed` is both the `after` target and the sequence predecessor of `Wait`, so the AC-23 event test cannot tell the `AwaitAnchor` edge from a sequence edge, although its doc says it can. Only the compensation test's `Enter` assertion guards the `EdgeKind::AwaitAnchor` filter, and that test asserts the node only as not the await and not `Armed`, not as the forward body of `Full`. | tests/tc_154_settle_deadline_captures.rs:995-1028 |

## Dispositions

Round 1, fix commit 138d713 (reviewed as `git diff 98e95d0..138d713`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 138d713: AC-13 adds the fixed-sample u64 edge and AC-20 asserts the window counts from the index with epoch 1_000_000_007 |
| FND-002 | fixed | 138d713: `AwaitDeclaration.after` is read from QSL's `AwaitAnchor` causal edge into the await's Enter port; FR-100 Inputs names both kinds |

Round 2, fix commit 753aabe (reviewed as `git diff 138d713..753aabe`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-003 | fixed | 753aabe: the AC-23 event test asserts port `Exit` on the node named `Armed`; new `a_compensation_anchor_is_the_entry_of_the_forward_body` asserts port `Enter` over the new `Awaiting::after_compensation()` fixture |

Round 3, fix commit 3ae1f09 (reviewed as `git diff 753aabe..3ae1f09`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | 3ae1f09: the event fixture puts `Spacer` between `Armed` and `Wait`, so the AC-23 event test's `Armed` assertion now separates the anchor edge from the sequence edge; the compensation test asserts the anchor node is `Applied` at port `Enter` |

Round 4, fix commit 8aadf41 (reviewed as `git diff 38f379b..8aadf41`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-002 | deferred | QSL-1875: 8aadf41 removes the `AwaitDeclaration.after` read because this repo forbids naming causal edges (tc_150 `no_source_constructs_a_causal_edge`) and QSL exposes no public accessor for the await's `after` {kind, target}; FR-100 Inputs now says so and cites QSL-1875 and STD-659 |
| FND-003 | deferred | QSL-1875: no longer applicable at 8aadf41, which removes FR-100-AC-23 and the event test; it applies again when the anchor read returns with QSL-1875 |
| FND-004 | deferred | QSL-1875: no longer applicable at 8aadf41, which removes the compensation test, the `after_compensation` fixture and the `Spacer` node; it applies again when the anchor read returns with QSL-1875 |
