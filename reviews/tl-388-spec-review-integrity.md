---
id: SR-2127053
title: "TL-388 PR #150 spec delta: integrity"
type: SpecReview
analysis: integrity
review_set: subset
scope: "agent-ix/quire-protocol; TL-388; PR #150 head 98e95d0; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md, spec/test-cases/TC-154-settle-deadline-branches-and-bind-captures.md, spec/tests.md (spec delta only)"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
---
## Summary

Ticket: TL-388. Consistency of the FR-100 delta with itself, with TC-154 and spec/tests.md, and with the QSpec owner requirements it cites (FR-061, FR-241, FR-242 read from quire-specification origin/main).

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-14 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-15 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-16 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-17 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-18 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- TC-154 (spec/test-cases/TC-154-settle-deadline-branches-and-bind-captures.md)

## Verdict

**FAIL.** TC-154's procedure and the spec/tests.md row agree with the new criteria, and the tests.md figure is no longer hand-kept. The no-`after` truth assignment contradicts the owner's definition of `pending`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The Behavior line "While the `after` observation is not admitted, absent selection is truth `pending` (FR-242) under completeness `complete` ..." and FR-100-AC-15 ("progress `closed` with completeness `complete` ... report truth `pending`") contradict QSpec FR-061's combination rule and FR-242's value table: `pending` requires activation `triggered` and decision-scope progress open. A wait whose `after` event is not observed is not triggered; under complete input and closed progress FR-061 gives activation `inactive` with truth `unavailable`, and with open progress activation-unknown with truth `unavailable`. FR-100 cites FR-061 for a value FR-061 forbids. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:75, 109 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The FR-100-AC-16, AC-17 and AC-18 rows were not rewritten. AC-16 still says "yields no superseding result" while its test now asserts a supersession. AC-17 still includes per-instance anchor independence and AC-18 still includes the earlier-progress-view refusal, but those halves are now tested only under AC-21 and AC-22, so the AC-17 and AC-18 tests no longer cover their rows. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:113-115 |

## Dispositions

Round 1, fix commit 138d713 (reviewed as `git diff 98e95d0..138d713`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | Behavior line 78 is fixed, but the FR-100-AC-15 row (line 112) still reads "report truth `pending`" for progress `closed` with completeness `complete`, contradicting the new Behavior line and the rewritten AC-15 test, which asserts `unavailable` |
