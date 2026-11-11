---
id: SR-2127055
title: "TL-388 PR #150 spec delta: scope-boundary"
type: SpecReview
analysis: scope-boundary
review_set: subset
scope: "agent-ix/quire-protocol; TL-388; PR #150 head 98e95d0; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md (spec delta only)"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
---
## Summary

Ticket: TL-388. Whether the amended FR-100 Inputs allocates the `after` anchor honestly between this library, QSL and QObs.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)

## Verdict

**PASS WITH FINDINGS.** The Inputs statement that QObs observations carry neither the checked event node nor the workflow instance is true of the owner's `observation::View` (quire-observation main), and allocating that attribution to the caller matches how response observations are already offered. That is acceptable for closing TL-388, whose ask was that the anchor instant come from an admitted observation instead of a caller-supplied number. What is missing is the QSL side: QSL already publishes which node the `after` names.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-100 Inputs does not say that QSL publishes the `after` anchor on the checked `await` as `after = {kind, target}` with kinds `event` and `compensation` (qsl-package protocol_view.rs), nor allocate reading it. As written, a reader concludes the library has no way to know which event is the `after` event, when only the observation-to-node attribution is missing. The `compensation` kind, whose anchor is the forward body's entry rather than an event observation, is not mentioned at all. State the QSL fact, allocate reading it to this library (or a ticketed follow-up), and name the observation-to-node attribution as the remaining owner gap. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:45, 49 |

## Dispositions

Round 1, fix commit 138d713 (reviewed as `git diff 98e95d0..138d713`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 138d713: Inputs states QSL's `after = {kind, target}`, the causal-relation port for both kinds, that this library reads it, and that observation-to-port attribution stays with the caller |

Round 4, fix commit 8aadf41 (reviewed as `git diff 38f379b..8aadf41`).

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | QSL-1875: FR-100 Inputs at 8aadf41 states that QSL publishes `after = {kind, target}` but no public accessor exposes it (QSL-1875) and that QObs views carry no node or workflow instance (STD-659), so the caller attributes the `after` observation; reading the anchor is deferred to QSL-1875 |
