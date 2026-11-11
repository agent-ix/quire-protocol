---
id: SR-2127034
title: "TL-9 FR-200..FR-204: EARS conformance"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/quire-protocol; PR #146 head dea0428; spec/functional/FR-200..FR-204, spec/test-cases/TC-200..TC-204, spec/tests.md"
review_set: subset
date: 2026-11-11
---

## Summary

Ticket: TL-9. Method: spec-review/ears-conformance. Spec-only PR; scope is `git diff origin/main...HEAD`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | FR-201's Description second sentence 'The dispatcher refuses a disposition whose retained lists or subjects differ from the request's.' is a declarative without SHALL; make it an event-driven SHALL clause (When ... the dispatcher SHALL refuse ...). | spec/functional/FR-201-bind-composition-premises-by-kind.md:18 |
| FND-002 | low | FR-202's Description ends 'No outcome value is derived from what it carries.' - passive, no responsible system and no SHALL NOT; restate as 'The library SHALL NOT derive an outcome value from ...'. | spec/functional/FR-202-carry-composition-and-refinement-results-unchanged.md:22 |

## Verdict

PASS WITH FINDINGS. FR-200, FR-203 and FR-204 Descriptions are well-formed event-driven EARS statements with one responsible system; Behavior bullets use 'shall'.

Units examined:

- `FR-200` (examined)
- `FR-201` (examined)
- `FR-202` (examined)
- `FR-203` (examined)
- `FR-204` (examined)

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | fef63ea (9fb5cea): When a composition disposition is returned, the dispatcher SHALL keep the request's environment, rely, liveness-dependency and fairness premises as four separate lists bound to their component subjects. The dispatcher SHALL refuse a disposition whose retained lists or subjects differ from the request's. |
| FND-002 | fixed | fef63ea (9fb5cea): When a composition or refinement backend returns a result, the library SHALL carry the paired counterexample, certificate, discharge record, witness or cause in the disposition with the same carry-unchanged semantics FR-301-AC-6 states for refinement. The library SHALL derive no outcome value from what it carries. |
