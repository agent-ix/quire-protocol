---
id: SR-2127020
title: "EARS conformance of TL-11 FR-050..FR-059"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-protocol; spec/functional/FR-050..FR-059, spec/test-cases/TC-050..TC-055, spec/tests.md (PR #143, head 00836cf)"
review_set: subset
date: 2026-11-11
---

## Summary

Ticket: TL-11. Engine check (quire 0.38.3, `quire validate --summary`) reports 10/10 docs grammar-clean, 0 findings. Descriptions use event-driven `When ..., the library SHALL` and FR-059 a correct state-driven `While ...`; Behavior bullets are ubiquitous `shall` statements. Plain-prose acceptance criteria match the repository convention (FR-043..FR-047 use the same form) and are outside the EARS lens, so they are not a finding. Semantic judgment finds one compound trigger and one non-singular bullet.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | The trigger joins two events: 'When QSL reports an `Unsupported` cause ... or the library meets a construct it cannot assess'. The second is not an event QSL reports and its response ('carries the QSL cause name unchanged') cannot apply to it. Split into one statement for a carried QSL cause and one for an assessment-time construct the library cannot judge (see scope-boundary FND-001). | FR-057; FR-054; FR-055; FR-059; spec/functional/FR-057-map-qsl-unsupported-outcomes-to-typed-dispositions.md:20 | wrong-requirement |
| FND-002 | low | One bullet carries two obligations: '... report a branch whose observed outcome differs as an outstanding violation. For a join with no authored rule it shall return the disposition of FR-057.' Split into two bullets so each maps to one criterion. | FR-055; spec/functional/FR-055-assess-join-settlement-per-policy.md:38 | wrong-requirement |

## Verdict

Findings above must be fixed or dispositioned before merge. Units examined:

- `FR-050` (examined)
- `FR-051` (examined)
- `FR-052` (examined)
- `FR-053` (examined)
- `FR-054` (examined)
- `FR-055` (examined)
- `FR-056` (examined)
- `FR-057` (examined)
- `FR-058` (examined)
- `FR-059` (examined)

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, commit 5ef24e3: When QSL reports an `Unsupported` cause for a protocol construct, the library SHALL return a typed `unsupported` disposition that names the construct. The library SHALL NOT approximate, truncate or repair the construct. |
| FND-002 | fixed | Round 1, commit 5ef24e3: - For a join of policy `any` or `quorum(n)` with no authored `outstanding` rule, the evaluator shall return the disposition of FR-057. |

## New findings (quire-protocol#147 spec delta)

Spec delta of quire-protocol#147 at ecee04fb29b155ea52a4c4f576882992d03f85da (`git diff b9c6e01..ecee04f -- spec/`): FR-050, FR-051, FR-053..FR-056, TC-050..TC-054, spec/tests.md.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | low | New FR-055 and FR-051 Behavior bullets each state two obligations in one sentence | spec/functional/FR-055-assess-join-settlement-per-policy.md:Behavior |
