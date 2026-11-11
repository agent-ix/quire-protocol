---
id: SR-2127047
title: "TL-6 PR #149 spec delta: failure domain"
type: SpecReview
analysis: failure-domain
review_set: subset
scope: "agent-ix/quire-protocol; TL-6; PR #149 head ecdac92; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md, spec/tests.md (spec delta only)"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
---
## Summary

Ticket: TL-6. Unstated failure modes in the amended FR-100: late contradictions, premise regression, and window edges.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-4 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)

## Verdict

**FAIL.** The delta adds a third outcome (no selection, unavailable) under closed/complete input but leaves the transitions into and out of it unspecified.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-100 states supersession only for a late observation. It is silent on a later progress or completeness record that contradicts a settled timeout (progress reopened, completeness incomplete or contradicted), and the implementation refuses that case. FR-033/QSpec FR-061 require an invalidating result. State it in FR-100 with an AC. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:54-55 |
| FND-002 | medium | AC-4 covers only a late *matching* response. With AC-7, a late false or valueless observation at or before the deadline also contradicts a settled timeout (the outcome becomes no selection, `unavailable`), but no AC requires the invalidating result for it, and the code emits none. Extend AC-4 or add an AC. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:64 |
| FND-003 | low | The authored wait is `within [lo, hi]`, but FR-100 speaks only of 'at or before the deadline'. A response observed before the window's lower bound is unaddressed, and the code accepts it as a response. State whether the window is closed on both ends. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:35 |

## Dispositions

Round 1, fix commit 183d4f1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 183d4f1 |
| FND-002 | fixed | 183d4f1 |
| FND-003 | fixed | 183d4f1 |
