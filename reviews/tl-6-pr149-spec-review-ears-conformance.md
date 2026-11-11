---
id: SR-2127046
title: "TL-6 PR #149 spec delta: EARS conformance"
type: SpecReview
analysis: ears-conformance
review_set: subset
scope: "agent-ix/quire-protocol; TL-6; PR #149 head ecdac92; spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md, spec/tests.md (spec delta only)"
relationships:
  - { target: ix://agent-ix/quire-protocol/FR-100, type: reviews }
---
## Summary

Ticket: TL-6. EARS conformance of the new FR-100 Behavior bullet and FR-100-AC-7.

## Examined

- FR-100 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)
- FR-100-AC-7 (spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md)

## Verdict

**PASS WITH FINDINGS.** The new bullet is a well-formed event-driven ('When ...') statement with one SHALL, matching the one-shall-per-bullet convention. The term 'has no value' does not match what the code maps to it.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | 'valueless' / 'has no value' is undefined, and the implementation (`Constraint::of`, src/deadline.rs:199-210) also treats `NotBoolean(value)` — a constraint that has a value, just not a Boolean — and a family result as valueless. Two implementers would read this differently. Define the term by the QSL slot outcomes it covers (unavailable, undefined, refused, incomplete, not Boolean, family result). | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:67 |
| FND-002 | low | The bullet's single SHALL carries two responses ('select no branch and report truth `unavailable` with execution `completed`'), and 'select no branch' states a prohibition as an action. Prefer 'SHALL NOT select either branch' with the reported dispositions in Outputs. | spec/functional/FR-100-settle-inclusive-deadline-branches-on-complete-progress.md:52 |

## Dispositions

Round 1, fix commit 183d4f1.

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | 183d4f1 |
| FND-002 | fixed | 183d4f1 |
