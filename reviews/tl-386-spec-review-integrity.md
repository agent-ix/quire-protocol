---
id: SR-2127066
title: "TL-386 PR #153: spec-review integrity"
type: SpecReview
analysis: integrity
scope: "agent-ix/quire-protocol; PR #153 head 3269b80; spec/functional/FR-019, spec/test-cases/TC-003, spec/tests.md"
review_set: subset
date: 2026-11-12
---

## Summary

Ticket: TL-386. Method: spec-review integrity sub-analysis of the small FR-019 edit; the EARS conformance of the edited statement was checked in the same pass.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | The TC-003 procedure tells the tester to substitute choice-visibility evidence "(owner and visible set)". FR-019-AC-2 names "(choice, owner or visible set)". The test case omits the choice member that the criterion it verifies names (see gap-analysis SR-2127065 FND-001). | spec/test-cases/TC-003-assess-compensation.md:21 |
| FND-002 | low | The FR-019 Inputs paragraph now lists choice-visibility evidence as carried by the binding, but its closing sentence ("A caller-built obligation label, control coordinate, trigger identity, capture or anchor is not a production input") does not mention it. That leaves open whether caller-built visibility evidence is an acceptable input. The edited line 44 also breaks the paragraph's wrap width. | spec/functional/FR-019-derive-obligation-activation-state.md:44-46 |

## Verdict

PASS WITH FINDINGS (two low). The edited statement is still an EARS event-driven requirement ("When ... is substituted, the evaluator SHALL refuse the activation binding"), and it is consistent with AC-2 and with the FR-479 relationship already declared in the frontmatter. The spec/tests.md row keeps its AC range and TC-003 binding, and its note is accurate apart from the accessor clause raised in SR-2127065 FND-002. No new relationship edges were added, so dependency analysis does not apply.

Units examined:

- `FR-019` (examined)
- `FR-019-AC-2` (examined)
- `TC-003` (examined)
- `spec/tests.md FR-019 row` (examined)

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | fc4a824 TC-003: "workflow, role, effect, predicate, capture, choice-visibility evidence (choice, owner or visible set), and evaluation-anchor identities." |
| FND-002 | fixed | fc4a824 FR-019: "immutable captures, choice-visibility evidence (`FR-479`) and evaluation anchor. Closure and completeness remain owner-issued facts. A caller-built obligation label, control coordinate, trigger identity, capture, choice-visibility evidence or anchor is not a production input." (paragraph re-wrapped) |
