---
id: SR-2127054
title: "TL-388 PR #150 spec delta: ears-conformance"
type: SpecReview
analysis: ears-conformance
review_set: subset
scope: "agent-ix/quire-protocol; TL-388; PR #150 head 98e95d0; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md (spec delta only)"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
---
## Summary

Ticket: TL-388. EARS form of the requirement statements and Behavior bullets FR-100 adds or edits, and atomicity of AC-14..AC-18.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-14 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-15 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-16 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-17 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-18 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)

## Verdict

**PASS WITH FINDINGS.** Six of the seven new Behavior lines are single-`SHALL` EARS statements (ubiquitous, event-driven `When`, or unwanted-behaviour refusals). One new line carries no `SHALL`, and two new criteria each pack two independent outcomes.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The new Behavior line "While the `after` observation is not admitted, absent selection is truth `pending` ... and truth `unavailable` ..." has no `SHALL` and states two outcomes; write it as one `While ..., the evaluator SHALL report ...` statement per completeness case. The closing paragraph "No timeout or response can be selected ..., so its late admission supersedes nothing under FR-033" is likewise normative content without a `SHALL`. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:75, 82 |
| FND-002 | low | FR-100-AC-17 joins the second-`after` refusal with per-instance anchor independence, and FR-100-AC-18 joins the clock-family refusal with the earlier-progress-view refusal; each is two independent pass/fail outcomes in one criterion. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:111-112 |
