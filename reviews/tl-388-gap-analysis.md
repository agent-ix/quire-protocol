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
