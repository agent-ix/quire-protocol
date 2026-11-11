---
id: SR-2127036
title: "TL-9 FR-200..FR-204: dependency"
type: SpecReview
analysis: dependency
scope: "agent-ix/quire-protocol; PR #146 head dea0428; spec/functional/FR-200..FR-204, spec/test-cases/TC-200..TC-204, spec/tests.md"
review_set: subset
date: 2026-11-11
---

## Summary

Ticket: TL-9. Method: spec-review/dependency. Spec-only PR; scope is `git diff origin/main...HEAD`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-202 carries composition results but its relationships and Dependencies omit QSpec FR-489 (whose Outputs define the composition result as an FR-360 result plus the premise discharge record) and FR-360 (the owner table mapping each label to its FR-241/FR-242/FR-243 combination). These are the owner contracts the axis carrying needs; see spec-review/base FND-001. | spec/functional/FR-202-carry-composition-and-refinement-results-unchanged.md:5 |

## Verdict

PASS WITH FINDINGS. Every Dependencies bullet in FR-200, FR-201, FR-203 and FR-204 has a matching relationships edge and vice versa (FR-008 as references, QSL FR-148 as references, FR-491 as references); no edge is dangling and no new edge creates a cycle with FR-301 or FR-057.

Units examined:

- `FR-200` (examined)
- `FR-201` (examined)
- `FR-202` (examined)
- `FR-203` (examined)
- `FR-204` (examined)

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | low | FR-202's Behavior and Dependencies now cite `ix://agent-ix/quire-specification/FR-243` (settlement basis), but its relationships carry no FR-243 edge; add `{ target: ix://agent-ix/quire-specification/FR-243, type: depends_on }`. | spec/functional/FR-202-carry-composition-and-refinement-results-unchanged.md:5 |

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | fef63ea (9fb5cea): - { target: ix://agent-ix/quire-specification/FR-489, type: depends_on } / - { target: ix://agent-ix/quire-specification/FR-360, type: depends_on } |
