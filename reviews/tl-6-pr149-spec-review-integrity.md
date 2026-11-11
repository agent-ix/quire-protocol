---
id: SR-2127045
title: "TL-6 PR #149 spec delta: integrity"
type: SpecReview
analysis: integrity
review_set: subset
scope: "agent-ix/quire-protocol; TL-6; PR #149 head ecdac92; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md, spec/tests.md (spec delta only)"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
---
## Summary

Ticket: TL-6. Integrity of the PR #149 spec delta: FR-100 Outputs rewording, the new Behavior bullet, FR-100-AC-7, and the spec/tests.md rows and counts.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-7 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- spec/tests.md#FR-100 (spec/tests.md)

## Verdict

**FAIL.** The new bullet and AC-7 agree with each other and with QSpec FR-049-AC-9 on 'no timeout', but FR-100's Description still mandates timeout for exactly that case, and spec/tests.md claims coverage the matrix does not compute. Recount from `quire matrix`: 475 criteria (matches); 203 tagged (doc says 204 passed); 38 method-without-symbol; 234 untagged. The 133 retired / 138 planned split cannot be derived from `quire matrix` (it emits no retired status); it is hand-maintained.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The Description's second SHALL selects timeout when progress is closed, completeness complete 'and no matching observation was admitted'. A false-constrained or valueless observation is not matching, so the Description requires timeout for exactly the case where the Behavior bullet, AC-3 and the new AC-7 forbid it. The PR amends this case and leaves the contradiction in the primary statement. Reword the Description condition (e.g. 'and no observation at or before the deadline was admitted'). | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:35 |
| FND-002 | high | spec/tests.md marks FR-100 '✅ Implemented — all 7 criteria have living traced tests' and TC-154 Implemented, and counts 204 passed / 138 planned, FR rows 11/31, TC rows 10/22, overall 27/60. `quire matrix` computes FR-100-AC-7 untagged and 203 tagged. The numbers become true only once an AC-7-tagged test lands; as committed they are false. The 133 retired figure is not derivable from `quire matrix`. | spec/tests.md:216,317,340-366 |
| FND-003 | low | 'In both of the completeness cases above' now follows the inserted AC-7 bullet, so 'above' spans three cases and 'both' no longer names its antecedents unambiguously. Name them: 'completeness `incomplete` or `contradicted`'. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:52-53 |

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-004 | low | Ruling on the retained hand count: delete it. `Retired 133` is a QSL-1549 history count that `quire matrix` cannot compute (it emits only tagged, untagged and method-without-symbol), so nothing checks it and it drifts with every edit. It records how a past stage went, not anything a reader acts on. Drop the Retired column and its prose, or replace it with a figure `quire matrix` derives. | spec/tests.md:359-365 |

## Dispositions

Round 1, fix commit 183d4f1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 183d4f1 |
| FND-002 | fixed | 183d4f1 |
| FND-003 | fixed | 183d4f1 |

Round 2, fix commit fb68003.

| FND | outcome | reason |
| --- | --- | --- |
| FND-004 | fixed | fb68003 |
