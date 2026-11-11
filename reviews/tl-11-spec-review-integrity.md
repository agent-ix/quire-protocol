---
id: SR-2127019
title: "Integrity analysis of TL-11 FR-050..FR-059"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-protocol; spec/functional/FR-050..FR-059, spec/test-cases/TC-050..TC-055, spec/tests.md (PR #143, head 00836cf)"
review_set: subset
date: 2026-11-11
---

## Summary

Ticket: TL-11. Completeness, consistency and atomicity of FR-050..FR-059. Statements are consistent with ADR-010 Decision 2 and 3 and with each other. Two Behavior lists contain a sentence broken mid-clause by the bullet split, which leaves a truncated obligation.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | medium | Bullet split broke one sentence into 'The evaluator shall report.' and 'The evaluator shall not repair, a violation (FR-027)'. The first is an obligation with no object; the second has a stray comma. Restore one statement: 'The evaluator shall report, and shall not repair, a violation (FR-027).' | FR-053; FR-027; spec/functional/FR-053-assess-role-lifetime-and-population.md:38 | wrong-requirement |
| FND-002 | medium | Same broken split as FR-053: 'The evaluator shall report.' followed by 'The evaluator shall not repair, a violation (FR-027).' Restore one statement. | FR-054; FR-027; spec/functional/FR-054-assess-queue-capacity-and-overflow-outcome.md:40 | wrong-requirement |

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
- `ADR-010` (context_only)

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | Round 1, commit 5ef24e3: - The evaluator shall report each violation without repairing it (FR-027). |
| FND-002 | fixed | Round 1, commit 5ef24e3: - The evaluator shall report each violation without repairing it (FR-027). |
| FND-003 | fixed | Round 2, df16bc9: FR-054-AC-9 \| A fact naming a node that is not a declared channel refuses the history with the node named, and no channel assessment is returned. FR-056-AC-6 \| A fact naming a node that is not a declared loop refuses the history with the node named. FR-055-AC-9/FR-056-AC-7: workflow identity refusals; FR-053 Behavior: refuse a spawn, retire or end fact for a static role; refuse a node that is no |
| FND-004 | fixed | Round 2, df16bc9: \| FR-053-AC-4 \| Over a static role, the assessment reports the bound as absent, reports no population violation and no lifetime violation for its activity, and does not infer a bound from the history. \| \| FR-053-AC-10 \| A spawn, retire or end fact for a static role refuses the history with the role named, and no role assessment is returned. \| |

## New findings (quire-protocol#147 spec delta)

Spec delta of quire-protocol#147 at ecee04fb29b155ea52a4c4f576882992d03f85da (`git diff b9c6e01..ecee04f -- spec/`): FR-050, FR-051, FR-053..FR-056, TC-050..TC-054, spec/tests.md.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-003 | medium | History refusals are specified for roles and joins but not for channels and loops (UndeclaredSubject, WorkflowNotText); FR-053's refusals have no Behavior line | spec/functional/FR-054-assess-queue-capacity-and-overflow-outcome.md:Behavior |
| FND-004 | low | FR-053-AC-4 is compound: static-role assessment and lifecycle-fact refusal in one criterion | spec/functional/FR-053-assess-role-lifetime-and-population.md:AC-4 |
