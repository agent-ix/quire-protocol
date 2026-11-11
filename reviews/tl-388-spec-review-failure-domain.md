---
id: SR-2127057
title: "TL-388 PR #150 spec delta: failure-domain"
type: SpecReview
analysis: failure-domain
review_set: subset
scope: "agent-ix/quire-protocol; TL-388; PR #150 head 98e95d0; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md (spec delta only)"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
---
## Summary

Ticket: TL-388. Unstated failure modes and identity confusion in the `after` anchor: absent, late, duplicated, other family, other clock. FR-033 (this repo) and QSpec FR-061 read for lineage.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-16 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-17 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-18 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)

## Verdict

**PASS WITH FINDINGS.** Absent, duplicated (AmbiguousAnchor) and other-family anchors are covered. It is right that a late `after` cannot supersede a timeout or a response, since neither can be selected without a window. Two failure modes remain: a late `after` against a result that rested on complete input, and an `after` on another clock.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | "No timeout or response can be selected before the `after` observation is admitted, so its late admission supersedes nothing under FR-033" justifies the absence of supersession by what was selected, but FR-033 keys supersession on a contradicted retained premise. A no-`after` result produced under completeness `complete` and progress `closed` rests on the premise that the input is complete, which a later admitted `after` observation contradicts (QSpec FR-061: a contradicted retained premise gets a new result naming the earlier one). FR-100-AC-16 then requires no superseding result in exactly that case. Either state why complete input does not cover the `after` event, or require an FR-033 result when the earlier no-`after` result was produced under complete input. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:76, 82, 110 |
| FND-002 | medium | FR-100 refuses an `after` observation of another clock family but not one on another clock of the same family, although the QObs view carries the clock identity and FR-100 already refuses a progress view on another clock. The window would then be counted on the wait's clock from an instant of a different clock. Add the refusal and a criterion. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:78, 112 |
