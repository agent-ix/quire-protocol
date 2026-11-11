---
id: SR-2127056
title: "TL-388 PR #150 spec delta: dependency"
type: SpecReview
analysis: dependency
review_set: subset
scope: "agent-ix/quire-protocol; TL-388; PR #150 head 98e95d0; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md (spec delta only)"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
---
## Summary

Ticket: TL-388. Whether the new FR-046 relationship and Dependencies bullet point at the owner of what FR-100 now reads.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-046 (spec/functional/FR-046-consume-temporal-evidence-through-the-evidence-types.md)

## Verdict

**FAIL.** No hidden enablement work, but the one new edge names the wrong requirement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-100 adds `references FR-046` and "FR-046 supplies the QObs observation view whose anchor is the instant of the `after` event", but this repo's FR-046 is "Consume temporal evidence through the in-memory evidence types" (QSL's `EvidenceRequest`/`EvidenceResult`) and says nothing about `quire_observation::authority::observation::View`. The view and its `AnchorRef` are QObs owner artifacts (the `quire.observation.record/v2` record over QSpec FR-288). Point the edge and the bullet at the QObs/QSpec owner requirement and drop FR-046. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:18, 118 |

## Dispositions

Round 1, fix commit 138d713 (reviewed as `git diff 98e95d0..138d713`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | still-open | The FR-046 relationship and Dependencies bullet are replaced by quire-observation FR-004 and FR-006, which are the right owners (QSpec FR-288 is the protocol-definition identity, so the review's FR-288 pointer was imprecise). But Inputs (line 51) still reads "the admitted observation of the awaited `after` event with its QObs anchor (FR-046)" |

Round 2, fix commit 753aabe (reviewed as `git diff 138d713..753aabe`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 753aabe: Inputs cites quire-observation FR-004 and FR-006; FR-100 no longer names FR-046 |
